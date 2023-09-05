use std::iter::Peekable;

struct IfStmt {
    expr: String,
    body: String,
}

struct IfChainStmt {
    ifs: Vec<IfStmt>,
    tail: Option<String>,
}

// Create engine used to evaluate template scripts.
pub fn new_engine() -> rhai::Engine {
    rhai::Engine::new()
}

// Expressions - @(<expr>)
// If - @if <expr> { <body> } [@elif  <expr> { <body> }]* [@else { <body> }]
pub fn parse(scope: &mut rhai::Scope, input: &str) -> String {
    let mut output = String::new();
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            // capture expression
            '@' if chars.peek().copied() == Some('(') => {
                chars.next(); // consume the '('

                let expr = chars.by_ref().take_while(|&c| c != ')').collect::<String>();

                // TODO: evaluate the expression
                output.push_str(&format!("[expr]({expr:?})"));
            }
            // capture keyword
            '@' if chars.peek().is_some_and(|c| c.is_alphabetic()) => {
                let keyword = capture_keyword(&mut chars);

                match keyword.as_str() {
                    "if" => {
                        let stmt = capture_if_chain_stmt(scope, &mut chars);

                        // TODO: evaluate if chain
                        for stmt in stmt.ifs {
                            output.push_str(&format!(
                                "[if]({expr:?}) {{ {body:?} }}",
                                expr = stmt.expr,
                                body = stmt.body
                            ));
                        }
                        if let Some(body) = stmt.tail {
                            output.push_str(&format!("[else] {{ {body:?} }}"));
                        }
                    }
                    // not a keyword
                    _ => {
                        output.push(c);
                        output.push_str(&keyword);
                        output.push(' ');
                    }
                }
            }
            _ => output.push(c),
        }
    }
    output
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
fn capture_body(scope: &mut rhai::Scope, chars: &mut impl Iterator<Item = char>) -> String {
    let body = chars.take_while(|&c| c != '}').collect::<String>();
    parse(scope, &body)
}

/// Assumes a preceeding '@if' or '@elif' has already been consumed.
/// Captures @if/@elif <expr> { <body> }.
fn capture_if_stmt(scope: &mut rhai::Scope, chars: &mut impl Iterator<Item = char>) -> IfStmt {
    let expr = chars.by_ref().take_while(|&c| c != '{').collect::<String>();
    let body = capture_body(scope, chars);
    IfStmt { expr, body }
}

/// Assumes a preceeding '@if' has already been consumed.
/// Captures @if <expr> { <body> } [@elif  <expr> { <body> }]* [@else { <body> }]
fn capture_if_chain_stmt(
    scope: &mut rhai::Scope,
    chars: &mut Peekable<impl Iterator<Item = char> + Clone>,
) -> IfChainStmt {
    let head = capture_if_stmt(scope, chars);

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

        let keyword = capture_keyword(chars);

        match keyword.as_str() {
            "elif" => {
                // no longer need to look ahead
                *chars = look_ahead;

                let elif = capture_if_stmt(scope, chars);
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

    stmt
}
