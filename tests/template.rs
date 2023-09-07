use flashmark::template::render;

#[test]
fn expression_string() {
    let mut scope = rhai::Scope::new();
    let input = "Hello, @(\"World\")!";
    let expected = "Hello, World!";
    let actual = render(&mut scope, input);
    assert_eq!(actual, expected);
}

#[test]
fn expression_arithmetic() {
    let mut scope = rhai::Scope::new();
    let input = "1 + 2 = @(1 + 2)";
    let expected = "1 + 2 = 3";
    let actual = render(&mut scope, input);
    assert_eq!(actual, expected);
}

#[test]
fn expression_variables() {
    let mut scope = rhai::Scope::new();
    scope.push("name", "World");
    let input = "Hello, @(name)!";
    let expected = "Hello, World!";
    let actual = render(&mut scope, input);
    assert_eq!(actual, expected);
}

#[test]
fn expression_variables_and_arithmetic() {
    let mut scope = rhai::Scope::new();
    scope.push("x", 1);
    scope.push("y", 2);
    let input = "@(x) + @(y) = @(x + y)";
    let expected = "1 + 2 = 3";
    let actual = render(&mut scope, input);
    assert_eq!(actual, expected);
}

#[test]
fn if_true_and_false() {
    let mut scope = rhai::Scope::new();
    let input = "@if true { true }";
    let expected = " true ";
    let actual = render(&mut scope, input);
    assert_eq!(actual, expected);

    let input = "@if false { true }";
    let expected = "";
    let actual = render(&mut scope, input);
    assert_eq!(actual, expected);
}

#[test]
fn if_else() {
    let mut scope = rhai::Scope::new();
    let input = "@if true { true } @else { false }";
    let expected = " true ";
    let actual = render(&mut scope, input);
    assert_eq!(actual, expected);

    let input = "@if false { true } @else { false }";
    let expected = " false ";
    let actual = render(&mut scope, input);
    assert_eq!(actual, expected);
}

#[test]
fn if_elif() {
    let mut scope = rhai::Scope::new();
    let input = "@if true { 1 } @elif true { 2 }";
    let expected = " 1 ";
    let actual = render(&mut scope, input);
    assert_eq!(actual, expected);

    let input = "@if false { 1 } @elif true { 2 }";
    let expected = " 2 ";
    let actual = render(&mut scope, input);
    assert_eq!(actual, expected);

    let input = "@if false { 1 } @elif false { 2 }";
    let expected = "";
    let actual = render(&mut scope, input);
    assert_eq!(actual, expected);
}

#[test]
fn if_elif_else() {
    let mut scope = rhai::Scope::new();
    let input = "@if true { 1 } @elif true { 2 } @else { 3 }";
    let expected = " 1 ";
    let actual = render(&mut scope, input);
    assert_eq!(actual, expected);

    let input = "@if false { 1 } @elif true { 2 } @else { 3 }";
    let expected = " 2 ";
    let actual = render(&mut scope, input);
    assert_eq!(actual, expected);

    let input = "@if false { 1 } @elif false { 2 } @else { 3 }";
    let expected = " 3 ";
    let actual = render(&mut scope, input);
    assert_eq!(actual, expected);
}

#[test]
fn if_expression() {
    let mut scope = rhai::Scope::new();
    scope.push("x", 1_i64);
    let input = "@if x == 1 { 1 } @else { 2 }";
    let expected = " 1 ";
    let actual = render(&mut scope, input);
    assert_eq!(actual, expected);

    let input = "@if x == 2 { 1 } @else { 2 }";
    let expected = " 2 ";
    let actual = render(&mut scope, input);
    assert_eq!(actual, expected);
}

#[test]
fn nested() {
    let mut scope = rhai::Scope::new();
    scope.push("name", "World");
    let input = "Hello, @(name)!\n@if true { Hello, @(name)! }";
    let expected = "Hello, World!\n Hello, World! ";
    let actual = render(&mut scope, input);
    assert_eq!(actual, expected);
}
