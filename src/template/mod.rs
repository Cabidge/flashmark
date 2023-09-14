pub mod evaluate;
pub mod parse;

pub fn render(input: &str) -> Vec<String> {
    let engine = rhai::Engine::new();
    render_with_engine(&engine, input)
}

pub fn render_with_engine(engine: &rhai::Engine, mut input: &str) -> Vec<String> {
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

    input
        .split("---\n")
        .map(|slide| render_slide(engine, &mut scope, slide))
        .collect()
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
