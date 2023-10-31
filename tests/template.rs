use indoc::indoc;

fn test_render(input: &str, expected: &str) {
    let actual = flashmark::template::render(input);
    assert_eq!(actual.trim_end(), expected.trim_end());
}

fn test_render_with_scope(scope: &mut rhai::Scope<'static>, input: &str, expected: &str) {
    use flashmark::template;
    let engine = template::new_engine();
    let env = template::Environment::new(&engine, scope);
    let actual = template::render_with_environment(env, input);

    assert_eq!(actual.trim_end(), expected.trim_end());
}

#[test]
fn front_matter() {
    test_render(
        indoc! {r#"
            ---
            let name = "World";
            ---
            Hello, @name!
        "#},
        "Hello, World!",
    );
}

#[test]
fn literal() {
    test_render("Hello, World!", "Hello, World!");
}

#[test]
fn literal_with_at() {
    test_render("Hello, \\@World!", "Hello, \\@World!");
}

#[test]
fn expression_string() {
    test_render(r#"Hello, @("World")!"#, "Hello, World!");
}

#[test]
fn expression_arithmetic() {
    test_render("1 + 2 = @(1 + 2)", "1 + 2 = 3");
}

#[test]
fn expression_variables() {
    let mut scope = rhai::Scope::new();
    scope.push("name", "World");

    test_render_with_scope(&mut scope, "Hello, @name!", "Hello, World!");
}

#[test]
fn expression_variables_and_arithmetic() {
    let mut scope = rhai::Scope::new();
    scope.push("x", 1);
    scope.push("y", 2);

    test_render_with_scope(&mut scope, "@x + @y = @(x + y)", "1 + 2 = 3");
}

#[test]
fn expression_function() {
    test_render(
        indoc! {"
            ---
            let sqr = |x| x * x;
            ---
            @x
            @(sqr.call(5))
        "},
        "25",
    );
}

#[test]
fn if_true_and_false() {
    test_render(
        indoc! {"
            @if true
                true
            @end
        "},
        "true",
    );
    test_render(
        indoc! {"
            @if false
                true
            @end
        "},
        "",
    );
}

#[test]
fn if_else() {
    test_render(
        indoc! {"
            @if true
                true
            @else
                false
            @end
        "},
        "true",
    );
    test_render(
        indoc! {"
            @if false
                true
            @else
                false
            @end
        "},
        "false",
    );
}

#[test]
fn if_elif() {
    test_render(
        indoc! {"
            @if true
                1
            @elif true
                2
            @end
        "},
        "1",
    );
    test_render(
        indoc! {"
            @if false
                1
            @elif true
                2
            @end
        "},
        "2",
    );
    test_render(
        indoc! {"
            @if false
                1
            @elif false
                2
            @end
        "},
        "",
    );
}

#[test]
fn if_elif_else() {
    test_render(
        indoc! {"
            @if true
                1
            @elif true
                2
            @else
                3
            @end
        "},
        "1",
    );
    test_render(
        indoc! {"
            @if false
                1
            @elif true
                2
            @else
                3
            @end
        "},
        "2",
    );
    test_render(
        indoc! {"
            @if false
                1
            @elif false
                2
            @else
                3
            @end
        "},
        "3",
    );
}

#[test]
fn if_expression() {
    let mut scope = rhai::Scope::new();
    scope.push("x", 1_i64);

    test_render_with_scope(
        &mut scope,
        indoc! {"
            @if x == 1
                1
            @else
                2
            @end
        "},
        "1",
    );
    test_render_with_scope(
        &mut scope,
        indoc! {"
            @if x == 2
                1
            @else
                2
            @end
        "},
        "2",
    );
}

#[test]
fn if_body_with_expression() {
    let mut scope = rhai::Scope::new();
    scope.push("name", "World");

    test_render_with_scope(
        &mut scope,
        indoc! {"
            Hello, @name!
            @if true
                Hello, @name!
            @end
        "},
        indoc! {"
            Hello, World!
            Hello, World!
        "},
    );
}

#[test]
fn for_array() {
    test_render(
        indoc! {"
            @for x in [1, 2, 3]
                @x
            @end
        "},
        "1\n2\n3",
    );
}

#[test]
fn for_expression() {
    let mut scope = rhai::Scope::new();
    let arr: rhai::Array = vec![1.into(), 2.into(), 3.into()];
    scope.push("arr", arr);

    test_render_with_scope(
        &mut scope,
        indoc! {"
            @for x in arr
                @x
            @end
        "},
        "1\n2\n3",
    );
}

#[test]
fn for_nested() {
    let mut scope = rhai::Scope::new();
    let arr: rhai::Array = vec![1.into(), 2.into(), 3.into()];
    scope.push("arr", arr);

    test_render_with_scope(
        &mut scope,
        indoc! {"
            @for x in arr
                @for y in arr
                    (@x, @y)
                @end
            @end
        "},
        "(1, 1)\n(1, 2)\n(1, 3)\n(2, 1)\n(2, 2)\n(2, 3)\n(3, 1)\n(3, 2)\n(3, 3)",
    );
}
