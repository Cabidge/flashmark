use rhai::packages::Package;

pub struct Environment {
    engine: rhai::Engine,
    scope: rhai::Scope<'static>,
    runtime: rhai::GlobalRuntimeState,
    funcs: Option<rhai::AST>,
}

pub type RhaiIterator = Box<dyn Iterator<Item = Result<rhai::Dynamic, Box<rhai::EvalAltResult>>>>;

impl Environment {
    pub fn new(
        engine: rhai::Engine,
        scope: rhai::Scope<'static>,
        funcs: Option<rhai::AST>,
    ) -> Self {
        // really messy code just to get the built-in iterators
        // TODO: find a better way to do this garbage
        let mut runtime = rhai::GlobalRuntimeState::new(&engine);
        runtime.push_import(
            "global",
            rhai::packages::StandardPackage::new().as_shared_module(),
        );

        Self {
            engine,
            scope,
            runtime,
            funcs,
        }
    }

    pub fn with_engine(engine: rhai::Engine) -> Self {
        Self::with_scope(engine, rhai::Scope::new())
    }

    pub fn with_scope(engine: rhai::Engine, scope: rhai::Scope<'static>) -> Self {
        Self::new(engine, scope, None)
    }

    pub fn try_with_script(
        engine: rhai::Engine,
        script: impl AsRef<str>,
    ) -> Result<Self, Box<rhai::EvalAltResult>> {
        let ast = engine.compile(script)?;

        let mut scope = rhai::Scope::new();
        engine.run_ast_with_scope(&mut scope, &ast)?;

        let funcs = ast.has_functions().then(|| ast.clone_functions_only());

        Ok(Self::new(engine, scope, funcs))
    }

    pub fn scope_mut(&mut self) -> &mut rhai::Scope<'static> {
        &mut self.scope
    }

    pub fn compile_expr(&self, script: impl AsRef<str>) -> Result<rhai::AST, rhai::ParseError> {
        self.engine
            .compile_expression_with_scope(&self.scope, script)
    }

    pub fn eval_ast<T: rhai::Variant + Clone>(
        &mut self,
        ast: &rhai::AST,
    ) -> Result<T, Box<rhai::EvalAltResult>> {
        use std::borrow::Cow;

        let mut ast = Cow::Borrowed(ast);

        if let Some(funcs) = &self.funcs {
            ast = Cow::Owned(funcs.merge(&ast));
        }

        self.engine.eval_ast_with_scope(&mut self.scope, &ast)
    }

    pub fn get_iter(&self, value: rhai::Dynamic) -> Result<RhaiIterator, rhai::Dynamic> {
        if let Some(iter_fn) = self.runtime.get_iter(value.type_id()) {
            Ok(iter_fn(value))
        } else {
            Err(value)
        }
    }
}
