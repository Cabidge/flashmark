use std::fmt::Write;

use crate::template::parse::Stmt;

use super::{
    new_engine,
    parse::{Block, ForStmt, IfChainStmt, IfStmt},
};

pub struct Evaluator<'a, T: Write> {
    pub scope: &'a mut rhai::Scope<'static>,
    pub write: T,
}

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("Format Error: {0}")]
    Fmt(#[from] std::fmt::Error),
    #[error("Eval Error: {0}")]
    Eval(#[from] Box<rhai::EvalAltResult>),
    #[error("Parse Error: {0}")]
    Parse(#[from] rhai::ParseError),
}

impl<'a, T: Write> Evaluator<'a, T> {
    pub fn new(scope: &'a mut rhai::Scope<'static>, write: T) -> Self {
        Self { scope, write }
    }

    pub fn eval(&mut self, block: Block) -> Result<(), std::fmt::Error> {
        for res in block {
            let res = res
                .map_err(Error::Parse)
                .and_then(|stmt| self.eval_stmt(stmt));

            match res {
                Ok(()) => (),
                // failed to write, abort
                Err(Error::Fmt(err)) => return Err(err),
                Err(err) => write!(self.write, "{err}")?,
            }
        }

        Ok(())
    }

    fn eval_stmt(&mut self, stmt: Stmt) -> Result<(), Error> {
        match stmt {
            Stmt::Literal(lit) => self.write.write_str(&lit)?,
            Stmt::Expr(expr) => {
                let value = new_engine().eval_ast_with_scope::<rhai::Dynamic>(self.scope, &expr)?;
                write!(self.write, "{}", value)?;
            }
            Stmt::If(IfChainStmt { ifs, tail }) => {
                for IfStmt { expr, body } in ifs {
                    if new_engine().eval_ast_with_scope::<bool>(self.scope, &expr)? {
                        self.eval(body)?;
                        return Ok(());
                    }
                }

                if let Some(block) = tail {
                    self.eval(block)?;
                }
            }
            Stmt::For(ForStmt { name, expr, body }) => {
                let iter = new_engine().eval_ast_with_scope::<rhai::Array>(self.scope, &expr)?;

                let rewind_point = self.scope.len();
                self.scope.push(&name, rhai::Dynamic::UNIT);
                for value in iter {
                    self.scope.set_value(&name, value);
                    self.eval(body.clone())?;
                }
                self.scope.rewind(rewind_point);
            }
        }

        Ok(())
    }
}
