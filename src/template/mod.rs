pub mod evaluate;
pub mod parse;

struct ModuleResolver;

impl rhai::ModuleResolver for ModuleResolver {
    fn resolve(
        &self,
        _engine: &rhai::Engine,
        _source: Option<&str>,
        path: &str,
        pos: rhai::Position,
    ) -> Result<std::rc::Rc<rhai::Module>, Box<rhai::EvalAltResult>> {
        use rhai::packages::Package;

        match path {
            "rand" => Ok(rhai_rand::RandomPackage::new().as_shared_module()),
            _ => Err(rhai::EvalAltResult::ErrorModuleNotFound(path.into(), pos).into()),
        }
    }
}

pub fn new_engine() -> rhai::Engine {
    let mut engine = rhai::Engine::new();
    engine.set_module_resolver(ModuleResolver);

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
