pub mod evaluate;
pub mod parse;

// Create engine used to evaluate template scripts.
pub fn new_engine() -> rhai::Engine {
    rhai::Engine::new()
}

pub fn render_slide(
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
