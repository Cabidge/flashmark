pub mod evaluate;
pub mod parse;

pub fn new_engine() -> rhai::Engine {
    use rhai::packages::Package;

    let mut engine = rhai::Engine::new();
    engine.register_static_module("rand", rhai_rand::RandomPackage::new().as_shared_module());

    engine
}

pub fn render(input: &str) -> String {
    render_with_engine(&new_engine(), input)
}

pub fn render_with_engine(engine: &rhai::Engine, input: &str) -> String {
    let mut scope = rhai::Scope::new();

    let (front_matter, input) = parse::parse_front_matter(input);

    if let Some(front_matter) = front_matter {
        if let Err(err) = engine.run_with_scope(&mut scope, front_matter) {
            return format!("Error: {}", err);
        }
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
