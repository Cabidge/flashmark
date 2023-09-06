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
                let res = new_engine()
                    .eval_expression_with_scope::<rhai::Dynamic>(scope, &expr)
                    .map(|r| r.to_string())
                    .unwrap_or_else(|err| err.to_string());

                output.push_str(&res);
            }
            // capture keyword
            '@' if chars.peek().is_some_and(|c| c.is_alphabetic()) => {
                let keyword = capture_keyword(&mut chars);

                match keyword.as_str() {
                    "if" => {
                        let stmt = capture_if_chain_stmt(scope, &mut chars);

                        let engine = new_engine();
                        let try_else = 'check_ifs: {
                            for stmt in stmt.ifs {
                                match engine.eval_expression_with_scope::<bool>(scope, &stmt.expr) {
                                    Ok(true) => {
                                        output.push_str(&stmt.body);
                                        break 'check_ifs false;
                                    }
                                    Ok(false) => continue,
                                    Err(err) => {
                                        output.push_str(&format!("Error: {}", err));
                                        break 'check_ifs false;
                                    }
                                }
                            }

                            true
                        };

                        if try_else {
                            if let Some(body) = stmt.tail {
                                output.push_str(&body);
                            }
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

        let keyword = capture_keyword(&mut look_ahead);

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

#[cfg(test)]
mod tests {
    #[test]
    fn expression_string() {
        let mut scope = rhai::Scope::new();
        let input = "Hello, @(\"World\")!";
        let expected = "Hello, World!";
        let actual = super::parse(&mut scope, input);
        assert_eq!(actual, expected);
    }

    #[test]
    fn expression_arithmetic() {
        let mut scope = rhai::Scope::new();
        let input = "1 + 2 = @(1 + 2)";
        let expected = "1 + 2 = 3";
        let actual = super::parse(&mut scope, input);
        assert_eq!(actual, expected);
    }

    #[test]
    fn expression_variables() {
        let mut scope = rhai::Scope::new();
        scope.push("name", "World");
        let input = "Hello, @(name)!";
        let expected = "Hello, World!";
        let actual = super::parse(&mut scope, input);
        assert_eq!(actual, expected);
    }

    #[test]
    fn expression_variables_and_arithmetic() {
        let mut scope = rhai::Scope::new();
        scope.push("x", 1);
        scope.push("y", 2);
        let input = "@(x) + @(y) = @(x + y)";
        let expected = "1 + 2 = 3";
        let actual = super::parse(&mut scope, input);
        assert_eq!(actual, expected);
    }

    #[test]
    fn if_true_and_false() {
        let mut scope = rhai::Scope::new();
        let input = "@if true { true }";
        let expected = " true ";
        let actual = super::parse(&mut scope, input);
        assert_eq!(actual, expected);

        let input = "@if false { true }";
        let expected = "";
        let actual = super::parse(&mut scope, input);
        assert_eq!(actual, expected);
    }

    #[test]
    fn if_else() {
        let mut scope = rhai::Scope::new();
        let input = "@if true { true } @else { false }";
        let expected = " true ";
        let actual = super::parse(&mut scope, input);
        assert_eq!(actual, expected);

        let input = "@if false { true } @else { false }";
        let expected = " false ";
        let actual = super::parse(&mut scope, input);
        assert_eq!(actual, expected);
    }

    #[test]
    fn if_elif() {
        let mut scope = rhai::Scope::new();
        let input = "@if true { 1 } @elif true { 2 }";
        let expected = " 1 ";
        let actual = super::parse(&mut scope, input);
        assert_eq!(actual, expected);

        let input = "@if false { 1 } @elif true { 2 }";
        let expected = " 2 ";
        let actual = super::parse(&mut scope, input);
        assert_eq!(actual, expected);

        let input = "@if false { 1 } @elif false { 2 }";
        let expected = "";
        let actual = super::parse(&mut scope, input);
        assert_eq!(actual, expected);
    }

    #[test]
    fn if_elif_else() {
        let mut scope = rhai::Scope::new();
        let input = "@if true { 1 } @elif true { 2 } @else { 3 }";
        let expected = " 1 ";
        let actual = super::parse(&mut scope, input);
        assert_eq!(actual, expected);

        let input = "@if false { 1 } @elif true { 2 } @else { 3 }";
        let expected = " 2 ";
        let actual = super::parse(&mut scope, input);
        assert_eq!(actual, expected);

        let input = "@if false { 1 } @elif false { 2 } @else { 3 }";
        let expected = " 3 ";
        let actual = super::parse(&mut scope, input);
        assert_eq!(actual, expected);
    }

    #[test]
    fn if_expression() {
        let mut scope = rhai::Scope::new();
        scope.push("x", 1);
        let input = "@if x == 1 { 1 } @else { 2 }";
        let expected = " 1 ";
        let actual = super::parse(&mut scope, input);
        assert_eq!(actual, expected);

        let input = "@if x == 2 { 1 } @else { 2 }";
        let expected = " 2 ";
        let actual = super::parse(&mut scope, input);
        assert_eq!(actual, expected);
    }

    #[test]
    fn if_truthy() {
        let mut scope = rhai::Scope::new();
        scope.push("x", 1);
        let input = "@if x { 1 } @else { 2 }";
        let expected = " 1 ";
        let actual = super::parse(&mut scope, input);
        assert_eq!(actual, expected);

        let input = "@if 0 { 1 } @else { 2 }";
        let expected = " 2 ";
        let actual = super::parse(&mut scope, input);
        assert_eq!(actual, expected);
    }

    #[test]
    fn nested() {
        let mut scope = rhai::Scope::new();
        scope.push("name", "World");
        let input = "Hello, @(name)!\n@if true { Hello, @(name)! }";
        let expected = "Hello, World!\n Hello, World! ";
        let actual = super::parse(&mut scope, input);
        assert_eq!(actual, expected);
    }
}
