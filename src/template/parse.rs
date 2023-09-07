use std::iter::Peekable;

use super::new_engine;

pub type Block = Vec<Result<Stmt, rhai::ParseError>>;

pub enum Stmt {
    Expr(rhai::AST),
    If(IfChainStmt),
    Literal(String),
}

pub struct IfStmt {
    pub expr: rhai::AST,
    pub body: Block,
}

pub struct IfChainStmt {
    pub ifs: Vec<IfStmt>,
    pub tail: Option<Block>,
}

// Expressions - @(<expr>)
// If - @if <expr> { <body> } [@elif  <expr> { <body> }]* [@else { <body> }]
pub fn parse(scope: &mut rhai::Scope, input: &str) -> Block {
    let mut block = vec![];

    let mut literal = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            // capture expression
            '@' if chars.peek().copied() == Some('(') => {
                block.push(Ok(Stmt::Literal(std::mem::take(&mut literal)))); // push literal to block

                chars.next(); // consume the '('

                let expr = chars.by_ref().take_while(|&c| c != ')').collect::<String>();

                match new_engine().compile_expression_with_scope(scope, expr) {
                    Ok(ast) => block.push(Ok(Stmt::Expr(ast))),
                    Err(err) => block.push(Err(err)),
                }
            }
            // capture keyword
            '@' if chars.peek().is_some_and(|c| c.is_alphabetic()) => {
                let keyword = capture_keyword(&mut chars);

                match keyword.as_str() {
                    "if" => {
                        block.push(Ok(Stmt::Literal(std::mem::take(&mut literal)))); // push literal to block

                        match capture_if_chain_stmt(scope, &mut chars) {
                            Ok(stmt) => block.push(Ok(Stmt::If(stmt))),
                            Err(err) => block.push(Err(err)),
                        }
                    }
                    // not a keyword
                    _ => {
                        literal.push(c);
                        literal.push_str(&keyword);
                        literal.push(' ');
                    }
                }
            }
            _ => literal.push(c),
        }
    }

    if !literal.is_empty() {
        block.push(Ok(Stmt::Literal(literal)));
    }

    block
}

/// Assumes the opening '@' has already been consumed.
/// Captures @<keyword>.
fn capture_keyword(chars: &mut Peekable<impl Iterator<Item = char>>) -> String {
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
fn capture_body(scope: &mut rhai::Scope, chars: &mut impl Iterator<Item = char>) -> Block {
    let body = chars.take_while(|&c| c != '}').collect::<String>();
    parse(scope, &body)
}

/// Assumes a preceeding '@if' or '@elif' has already been consumed.
/// Captures @if/@elif <expr> { <body> }.
fn capture_if_stmt(
    scope: &mut rhai::Scope,
    chars: &mut impl Iterator<Item = char>,
) -> Result<IfStmt, rhai::ParseError> {
    let expr = chars.by_ref().take_while(|&c| c != '{').collect::<String>();
    let expr = new_engine().compile_expression_with_scope(scope, expr)?;

    let body = capture_body(scope, chars);

    Ok(IfStmt { expr, body })
}

/// Assumes a preceeding '@if' has already been consumed.
/// Captures @if <expr> { <body> } [@elif  <expr> { <body> }]* [@else { <body> }]
fn capture_if_chain_stmt(
    scope: &mut rhai::Scope,
    chars: &mut Peekable<impl Iterator<Item = char> + Clone>,
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
