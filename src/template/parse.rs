use std::iter::Peekable;

use super::new_engine;

pub type Block = Vec<Result<Stmt, rhai::ParseError>>;

pub struct Parser<'a> {
    pub scope: &'a rhai::Scope<'static>,
    pub chars: CharStream<'a>,
    current_state: Option<ParserState>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParserState {
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
            scope,
            chars: input.chars().peekable(),
            current_state: Some(ParserState::Literal(String::new())),
        }
    }

    pub fn parse_stmt(&mut self) -> Option<Result<Stmt, rhai::ParseError>> {
        let current_state = self.current_state.take();
        let (res, next_state) = Self::step(current_state, self.scope, &mut self.chars);

        self.current_state = next_state;

        res
    }

    fn step(
        current_state: Option<ParserState>,
        scope: &rhai::Scope<'static>,
        chars: &mut CharStream,
    ) -> (Option<Result<Stmt, rhai::ParseError>>, Option<ParserState>) {
        let Some(current_state) = current_state else {
            return (None, None);
        };

        match current_state {
            ParserState::Literal(mut literal) => {
                if let Some(c) = chars.next() {
                    match c {
                        // capture expression
                        '@' if chars.peek().copied() == Some('(') => {
                            (Some(Ok(Stmt::Literal(literal))), Some(ParserState::Expr))
                        }
                        // capture keyword
                        '@' if chars.peek().is_some_and(|c| c.is_alphabetic()) => {
                            let keyword = capture_keyword(chars);

                            match keyword.as_str() {
                                "if" => (Some(Ok(Stmt::Literal(literal))), Some(ParserState::If)),
                                _ => {
                                    literal.push(c);
                                    literal.push_str(&keyword);
                                    literal.push(' ');

                                    (None, Some(ParserState::Literal(literal)))
                                }
                            }
                        }
                        _ => {
                            literal.push(c);
                            (None, Some(ParserState::Literal(literal)))
                        }
                    }
                } else {
                    (Some(Ok(Stmt::Literal(literal))), None)
                }
            }
            ParserState::Expr => {
                chars.next(); // consume the '('

                let expr = chars.by_ref().take_while(|&c| c != ')').collect::<String>();

                let stmt = new_engine()
                    .compile_expression_with_scope(scope, expr)
                    .map(Stmt::Expr);

                (Some(stmt), Some(ParserState::Literal(String::new())))
            }
            ParserState::If => {
                let res = capture_if_chain_stmt(scope, chars).map(Stmt::If);
                (Some(res), Some(ParserState::Literal(String::new())))
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
            self.current_state.as_ref()?;

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
fn capture_body(scope: &rhai::Scope<'static>, chars: &mut CharStream) -> Block {
    let body = chars.take_while(|&c| c != '}').collect::<String>();
    parse(scope, &body)
}

/// Assumes a preceeding '@if' or '@elif' has already been consumed.
/// Captures @if/@elif <expr> { <body> }.
fn capture_if_stmt(
    scope: &rhai::Scope<'static>,
    chars: &mut CharStream,
) -> Result<IfStmt, rhai::ParseError> {
    let expr = chars.by_ref().take_while(|&c| c != '{').collect::<String>();
    let expr = new_engine().compile_expression_with_scope(scope, expr)?;

    let body = capture_body(scope, chars);

    Ok(IfStmt { expr, body })
}

/// Assumes a preceeding '@if' has already been consumed.
/// Captures @if <expr> { <body> } [@elif  <expr> { <body> }]* [@else { <body> }]
fn capture_if_chain_stmt(
    scope: &rhai::Scope<'static>,
    chars: &mut CharStream,
) -> Result<IfChainStmt, rhai::ParseError> {
    let head = capture_if_stmt(scope, chars)?;

    let mut stmt = IfChainStmt {
        ifs: vec![head],
        tail: None,
    };

    // look ahead to find elif or else
    loop {
        let mut look_ahead = (*chars).clone();

        if look_ahead.find(|c| !c.is_whitespace()) != Some('@') {
            break;
        }

        let keyword = capture_keyword(&mut look_ahead);

        match keyword.as_str() {
            "elif" => {
                // no longer need to look ahead
                *chars = look_ahead;

                let elif = capture_if_stmt(scope, chars)?;
                stmt.ifs.push(elif);
            }
            "else" => {
                // no longer need to look ahead
                *chars = look_ahead;

                // consume until '{'
                chars.by_ref().take_while(|&c| c != '{').for_each(drop);

                let body = capture_body(scope, chars);
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
