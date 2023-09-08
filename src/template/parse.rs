use std::iter::Peekable;

use super::new_engine;

pub type Block = Vec<Result<Stmt, rhai::ParseError>>;

pub struct Parser<'a> {
    pub state: ParserState<'a>,
    current_step: Option<ParserStep>,
}

pub struct ParserState<'a> {
    pub scope: &'a rhai::Scope<'static>,
    pub chars: CharStream<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParserStep {
    Literal(String),
    Expr,
    If,
}

pub enum Stmt {
    Expr(rhai::AST),
    If(IfChainStmt),
    Literal(String),
}

pub struct IfChainStmt {
    pub ifs: Vec<IfStmt>,
    pub tail: Option<Block>,
}

pub struct IfStmt {
    pub expr: rhai::AST,
    pub body: Block,
}

type CharStream<'a> = Peekable<std::str::Chars<'a>>;

impl<'a> Parser<'a> {
    pub fn new(scope: &'a rhai::Scope<'static>, input: &'a str) -> Self {
        Self {
            state: ParserState {
                scope,
                chars: input.chars().peekable(),
            },
            current_step: Some(ParserStep::Literal(String::new())),
        }
    }

    pub fn parse_stmt(&mut self) -> Option<Result<Stmt, rhai::ParseError>> {
        let current_state = self.current_step.take();
        let (res, next_state) = Self::step(current_state, &mut self.state);

        self.current_step = next_state;

        res
    }

    fn step(
        current_state: Option<ParserStep>,
        state: &mut ParserState,
    ) -> (Option<Result<Stmt, rhai::ParseError>>, Option<ParserStep>) {
        let Some(current_state) = current_state else {
            return (None, None);
        };

        match current_state {
            ParserStep::Literal(mut literal) => {
                let Some(c) = state.chars.next() else {
                    return (Some(Ok(Stmt::Literal(literal))), None);
                };

                match (c, state.chars.peek().copied()) {
                    // capture expression
                    ('@', Some('(')) => (Some(Ok(Stmt::Literal(literal))), Some(ParserStep::Expr)),
                    // capture keyword
                    ('@', Some(c_next)) if c_next.is_alphabetic() => {
                        let keyword = capture_keyword(&mut state.chars);

                        match keyword.as_str() {
                            "if" => (Some(Ok(Stmt::Literal(literal))), Some(ParserStep::If)),
                            _ => {
                                literal.push(c);
                                literal.push_str(&keyword);
                                literal.push(' ');

                                (None, Some(ParserStep::Literal(literal)))
                            }
                        }
                    }
                    _ => {
                        literal.push(c);
                        (None, Some(ParserStep::Literal(literal)))
                    }
                }
            }
            ParserStep::Expr => {
                state.chars.next(); // consume the '('

                let expr = state
                    .chars
                    .by_ref()
                    .take_while(|&c| c != ')')
                    .collect::<String>();

                let stmt = new_engine()
                    .compile_expression_with_scope(state.scope, expr)
                    .map(Stmt::Expr);

                (Some(stmt), Some(ParserStep::Literal(String::new())))
            }
            ParserStep::If => {
                let res = capture_if_chain_stmt(state).map(Stmt::If);
                (Some(res), Some(ParserStep::Literal(String::new())))
            }
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<Stmt, rhai::ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        // keep parsing until we get a statement or run out of characters
        loop {
            // if state is None, we've run out of characters
            self.current_step.as_ref()?;

            if let Some(stmt) = self.parse_stmt() {
                return Some(stmt);
            }
        }
    }
}

// Expressions - @(<expr>)
// If - @if <expr> { <body> } [@elif  <expr> { <body> }]* [@else { <body> }]
pub fn parse(scope: &rhai::Scope<'static>, input: &str) -> Block {
    Parser::new(scope, input).collect()
}

/// Assumes the opening '@' has already been consumed.
/// Captures @<keyword>.
fn capture_keyword(chars: &mut CharStream) -> String {
    let mut keyword = String::new();
    while let Some(c) = chars.next_if(|c| c.is_alphabetic()) {
        keyword.push(c);
    }
    keyword
}

/// Assumes the opening '{' has already been consumed.
/// Captures { <body> }.
///
/// Returns the parsed body.
fn capture_body(state: &mut ParserState) -> Block {
    let body = state
        .chars
        .by_ref()
        .take_while(|&c| c != '}')
        .collect::<String>();

    parse(state.scope, &body)
}

/// Assumes a preceeding '@if' or '@elif' has already been consumed.
/// Captures @if/@elif <expr> { <body> }.
fn capture_if_stmt(state: &mut ParserState) -> Result<IfStmt, rhai::ParseError> {
    let expr = state
        .chars
        .by_ref()
        .take_while(|&c| c != '{')
        .collect::<String>();
    let expr = new_engine().compile_expression_with_scope(state.scope, expr)?;

    let body = capture_body(state);

    Ok(IfStmt { expr, body })
}

/// Assumes a preceeding '@if' has already been consumed.
/// Captures @if <expr> { <body> } [@elif  <expr> { <body> }]* [@else { <body> }]
fn capture_if_chain_stmt(state: &mut ParserState) -> Result<IfChainStmt, rhai::ParseError> {
    let head = capture_if_stmt(state)?;

    let mut stmt = IfChainStmt {
        ifs: vec![head],
        tail: None,
    };

    // look ahead to find elif or else
    loop {
        let mut look_ahead = state.chars.clone();

        if look_ahead.find(|c| !c.is_whitespace()) != Some('@') {
            break;
        }

        let keyword = capture_keyword(&mut look_ahead);

        match keyword.as_str() {
            "elif" => {
                // no longer need to look ahead
                state.chars = look_ahead;

                let elif = capture_if_stmt(state)?;
                stmt.ifs.push(elif);
            }
            "else" => {
                // no longer need to look ahead
                state.chars = look_ahead;

                // consume until '{'
                state
                    .chars
                    .by_ref()
                    .take_while(|&c| c != '{')
                    .for_each(drop);

                let body = capture_body(state);
                stmt.tail = Some(body);

                // no more elif or else
                break;
            }
            _ => break,
        }
    }

    Ok(stmt)
}

#[cfg(test)]
mod tests {}
