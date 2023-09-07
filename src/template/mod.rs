pub mod evaluate;
pub mod parse;

// Create engine used to evaluate template scripts.
pub fn new_engine() -> rhai::Engine {
    rhai::Engine::new()
}

pub fn render(scope: &mut rhai::Scope, input: &str) -> String {
    let block = parse::parse(scope, input);
    evaluate::eval(scope, block)
}
