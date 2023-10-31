use rhai::packages::Package;

pub struct Environment<'a> {
    engine: &'a rhai::Engine,
    scope: &'a mut rhai::Scope<'static>,
    runtime: rhai::GlobalRuntimeState,
}

pub type RhaiIterator = Box<dyn Iterator<Item = Result<rhai::Dynamic, Box<rhai::EvalAltResult>>>>;

impl<'a> Environment<'a> {
    pub fn new(engine: &'a rhai::Engine, scope: &'a mut rhai::Scope<'static>) -> Self {
        // really messy code just to get the built-in iterators
        // TODO: find a better way to do this garbage
        let mut runtime = rhai::GlobalRuntimeState::new(engine);
        runtime.push_import(
            "global",
            rhai::packages::StandardPackage::new().as_shared_module(),
        );

        Self {
            engine,
            scope,
            runtime,
        }
    }

    pub fn scope_mut(&mut self) -> &mut rhai::Scope<'static> {
        self.scope
    }

    pub fn compile_expr(&self, script: impl AsRef<str>) -> Result<rhai::AST, rhai::ParseError> {
        self.engine
            .compile_expression_with_scope(self.scope, script)
    }

    pub fn eval_ast<T: rhai::Variant + Clone>(
        &mut self,
        ast: &rhai::AST,
    ) -> Result<T, Box<rhai::EvalAltResult>> {
        self.engine.eval_ast_with_scope(self.scope, ast)
    }

    pub fn get_iter(&self, value: rhai::Dynamic) -> Result<RhaiIterator, rhai::Dynamic> {
        if let Some(iter_fn) = self.runtime.get_iter(value.type_id()) {
            Ok(iter_fn(value))
        } else {
            Err(value)
        }
    }
}
