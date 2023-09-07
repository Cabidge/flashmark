use std::iter::Peekable;

use super::new_engine;

pub type Block = Vec<Result<Stmt, rhai::ParseError>>;

pub struct Parser<'a> {
    pub scope: &'a rhai::Scope<'static>,
    pub chars: CharStream<'a>,
    current_state: ParserState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParserState {
    Literal(String),
    Expr,
    If,
    End,
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
            current_state: ParserState::Literal(String::new()),
        }
    }

    pub fn parse_stmt(&mut self) -> Option<Result<Stmt, rhai::ParseError>> {
        match self.current_state {
            ParserState::Literal(ref mut literal) => {
                if let Some(c) = self.chars.next() {
                    match c {
                        // capture expression
                        '@' if self.chars.peek().copied() == Some('(') => {
                            let literal = std::mem::take(literal);
                            self.current_state = ParserState::Expr;
                            Some(Ok(Stmt::Literal(literal)))
                        }
                        // capture keyword
                        '@' if self.chars.peek().is_some_and(|c| c.is_alphabetic()) => {
                            let keyword = capture_keyword(&mut self.chars);

                            match keyword.as_str() {
                                "if" => {
                                    let stmt = Stmt::Literal(std::mem::take(literal));

                                    self.current_state = ParserState::If;
                                    Some(Ok(stmt))
                                }
                                _ => {
                                    literal.push(c);
                                    literal.push_str(&keyword);
                                    literal.push(' ');
                                    None
                                }
                            }
                        }
                        _ => {
                            literal.push(c);
                            self.parse_stmt()
                        }
                    }
                } else {
                    let literal = std::mem::take(literal);
                    self.current_state = ParserState::End;
                    Some(Ok(Stmt::Literal(literal)))
                }
            }
            ParserState::Expr => {
                self.chars.next(); // consume the '('

                let expr = self
                    .chars
                    .by_ref()
                    .take_while(|&c| c != ')')
                    .collect::<String>();

                let stmt = new_engine()
                    .compile_expression_with_scope(self.scope, expr)
                    .map(Stmt::Expr);

                self.current_state = ParserState::Literal(String::new());

                Some(stmt)
            }
            ParserState::If => {
                let res = capture_if_chain_stmt(self.scope, &mut self.chars).map(Stmt::If);
                self.current_state = ParserState::Literal(String::new());
                Some(res)
            }
            ParserState::End => None,
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<Stmt, rhai::ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        // keep parsing until we get a statement or run out of characters
        while self.current_state != ParserState::End {
            if let Some(stmt) = self.parse_stmt() {
                return Some(stmt);
            }
        }

        None
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
