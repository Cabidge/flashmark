pub mod environment;
pub mod parse;

pub use environment::Environment;

use parse::*;

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
    let mut env = Environment::new(engine, scope);

    let mut output = String::new();
    parse_root(&env, &mut input.lines()).render(&mut env, 0, &mut output);

    output
}

fn unindent(line: &str, amount: usize) -> &str {
    if line.len() <= amount {
        return line.trim_start();
    }

    let (prefix, rest) = line.split_at(amount);

    if prefix.trim().is_empty() {
        rest
    } else {
        line.trim_start()
    }
}

impl<'a> Block<'a> {
    fn render(&self, env: &mut Environment<'_>, unindent_amount: usize, output: &mut String) {
        let inner_unindent = self.min_indentation().saturating_sub(self.indent);

        let unindent_amount = unindent_amount + inner_unindent;

        for node in self.nodes.iter() {
            node.render(env, unindent_amount, output);
        }
    }
}

impl<'a> IfChainBlock<'a> {
    fn render(&self, env: &mut Environment<'_>, unindent_amount: usize, output: &mut String) {
        let Some(block) = self.get_branch(env) else {
            return;
        };

        block.render(env, unindent_amount, output);
    }
}

impl<'a> ForBlock<'a> {
    fn render(&self, env: &mut Environment<'_>, unindent_amount: usize, output: &mut String) {
        let iterable = env.eval_ast(&self.iterable).unwrap();
        let iterator = env.get_iter(iterable).unwrap();

        for item in iterator {
            env.scope_mut().push(self.binding, item.unwrap());

            self.block.render(env, unindent_amount, output);

            env.scope_mut().pop();
        }
    }
}

impl<'a> Line<'a> {
    fn render(&self, env: &mut Environment<'_>, unindent_amount: usize, output: &mut String) {
        let unindented = unindent(self.front, unindent_amount);
        output.push_str(unindented);

        for (expr, text) in &self.expressions {
            use std::fmt::Write;

            let value = env.eval_ast::<rhai::Dynamic>(expr).unwrap();
            write!(output, "{}", value).expect("writing to string can't fail");

            output.push_str(text);
        }
        output.push('\n');
    }
}

impl<'a> Node<'a> {
    fn render(&self, env: &mut Environment<'_>, unindent_amount: usize, output: &mut String) {
        match self {
            Node::Line(line) => line.render(env, unindent_amount, output),
            Node::If(if_block) => if_block.render(env, unindent_amount, output),
            Node::For(for_block) => for_block.render(env, unindent_amount, output),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod node_indentation {
        use super::*;

        fn new_line(s: &str) -> Node<'_> {
            Node::Line(Line {
                front: s,
                expressions: Vec::new(),
            })
        }

        #[test]
        fn line() {
            let node = new_line("  hello");
            assert_eq!(node.indentation(), Some(2));
        }

        #[test]
        fn empty_line() {
            let node = new_line("  ");
            assert_eq!(node.indentation(), None);
        }
    }

    mod unindent {
        use super::*;

        #[test]
        fn empty() {
            assert_eq!(unindent("", 2), "");
        }

        #[test]
        fn no_indent() {
            assert_eq!(unindent("hello", 2), "hello");
        }

        #[test]
        fn indent() {
            assert_eq!(unindent("  hello", 2), "hello");
        }

        #[test]
        fn extra_indent() {
            assert_eq!(unindent("    hello", 2), "  hello");
        }

        #[test]
        fn over_unindent() {
            assert_eq!(unindent("  hello", 10), "hello");
        }
    }
}
