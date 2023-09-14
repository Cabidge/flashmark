pub mod evaluate;
pub mod parse;

pub fn new_engine() -> rhai::Engine {
    rhai::Engine::new()
}

pub fn render(input: &str) -> String {
    render_with_engine(&new_engine(), input)
}

pub fn render_with_engine(engine: &rhai::Engine, mut input: &str) -> String {
    let mut scope = rhai::Scope::new();

    // check for front matter code block
    if let Some(stripped) = input.strip_prefix("---\n") {
        // TODO: handle invalid format
        let (front_matter, new_input) = stripped
            .split_once("---\n")
            .expect("front matter is not closed");

        input = new_input;

        // TODO: handle runtime error
        engine
            .run_with_scope(&mut scope, front_matter)
            .expect("failed to run front matter");
    }

    render_with_engine_and_scope(engine, &mut scope, input)
}

pub fn render_with_engine_and_scope(
    engine: &rhai::Engine,
    scope: &mut rhai::Scope<'static>,
    input: &str,
) -> String {
    let block = parse::Parser::new(engine, scope, input).collect();

    let mut output = String::new();

    let mut evaluator = evaluate::Evaluator::new(engine, scope, &mut output);
    evaluator.eval(block).unwrap();

    output
}
