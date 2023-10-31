pub mod environment;
pub mod parse;
pub mod render;

pub use environment::Environment;

use parse::*;
use render::Render;

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
    render_with_engine(new_engine(), input)
}

pub fn render_with_engine(engine: rhai::Engine, input: &str) -> String {
    let (front_matter, input) = parse::parse_front_matter(input);

    let env = if let Some(script) = front_matter {
        match Environment::try_with_script(engine, script) {
            Ok(env) => env,
            Err(err) => {
                return format!("Error: {}", err);
            }
        }
    } else {
        Environment::with_engine(engine)
    };

    render_with_environment(env, input)
}

pub fn render_with_environment(mut env: Environment, input: &str) -> String {
    let mut output = String::new();
    parse_root(&env, &mut input.lines()).render(&mut env, 0, &mut output);

    output
}
