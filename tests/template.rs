fn test_render(input: &str, expected: &str) {
    let actual = flashmark::template::render(input);
    assert_eq!(actual, expected);
}

fn test_render_with_scope(scope: &mut rhai::Scope<'static>, input: &str, expected: &str) {
    use flashmark::template;
    let actual = template::render_with_engine_and_scope(&template::new_engine(), scope, input);

    assert_eq!(actual, expected);
}

#[test]
fn literal() {
    test_render("Hello, World!", "Hello, World!");
}

#[test]
fn literal_with_at() {
    test_render("Hello, @World!", "Hello, @World!");
}

#[test]
fn expression_string() {
    test_render("Hello, @(\"World\")!", "Hello, World!");
}

#[test]
fn expression_arithmetic() {
    test_render("1 + 2 = @(1 + 2)", "1 + 2 = 3");
}

#[test]
fn expression_variables() {
    let mut scope = rhai::Scope::new();
    scope.push("name", "World");

    test_render_with_scope(&mut scope, "Hello, @(name)!", "Hello, World!");
}

#[test]
fn expression_variables_and_arithmetic() {
    let mut scope = rhai::Scope::new();
    scope.push("x", 1);
    scope.push("y", 2);

    test_render_with_scope(&mut scope, "@(x) + @(y) = @(x + y)", "1 + 2 = 3");
}

#[test]
fn if_true_and_false() {
    test_render("@if true { true }", " true ");
    test_render("@if false { true }", "");
}

#[test]
fn if_else() {
    test_render("@if true { true } @else { false }", " true ");
    test_render("@if false { true } @else { false }", " false ");
}

#[test]
fn if_elif() {
    test_render("@if true { 1 } @elif true { 2 }", " 1 ");
    test_render("@if false { 1 } @elif true { 2 }", " 2 ");
    test_render("@if false { 1 } @elif false { 2 }", "");
}

#[test]
fn if_elif_else() {
    test_render("@if true { 1 } @elif true { 2 } @else { 3 }", " 1 ");
    test_render("@if false { 1 } @elif true { 2 } @else { 3 }", " 2 ");
    test_render("@if false { 1 } @elif false { 2 } @else { 3 }", " 3 ");
}

#[test]
fn if_expression() {
    let mut scope = rhai::Scope::new();
    scope.push("x", 1_i64);

    test_render_with_scope(&mut scope, "@if x == 1 { 1 } @else { 2 }", " 1 ");
    test_render_with_scope(&mut scope, "@if x == 2 { 1 } @else { 2 }", " 2 ");
}

#[test]
fn if_newline() {
    test_render("@if true {\n    true\n}", "    true");
}

#[test]
fn if_body_with_expression() {
    let mut scope = rhai::Scope::new();
    scope.push("name", "World");

    test_render_with_scope(
        &mut scope,
        "Hello, @(name)!\n@if true { Hello, @(name)! }",
        "Hello, World!\n Hello, World! ",
    );
}

#[test]
fn for_array() {
    test_render("@for x in [1, 2, 3] { @(x) }", " 1  2  3 ");
}

#[test]
fn for_expression() {
    let mut scope = rhai::Scope::new();
    let arr: rhai::Array = vec![1.into(), 2.into(), 3.into()];
    scope.push("arr", arr);

    test_render_with_scope(&mut scope, "@for x in arr { @(x) }", " 1  2  3 ");
}

#[test]
fn for_nested() {
    let mut scope = rhai::Scope::new();
    let arr: rhai::Array = vec![1.into(), 2.into(), 3.into()];
    scope.push("arr", arr);

    test_render_with_scope(
        &mut scope,
        "@for x in arr {@for y in arr { (@(x), @(y)) }}",
        " (1, 1)  (1, 2)  (1, 3)  (2, 1)  (2, 2)  (2, 3)  (3, 1)  (3, 2)  (3, 3) ",
    );
}
