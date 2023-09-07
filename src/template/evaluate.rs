use crate::template::parse::Stmt;

use super::{
    new_engine,
    parse::{Block, IfChainStmt, IfStmt},
};

pub fn eval(scope: &mut rhai::Scope, block: Block) -> String {
    let mut output = String::new();

    for stmt in block {
        match stmt.map(|stmt| eval_stmt(scope, stmt)) {
            // parsed and evaluated
            Ok(Ok(rendered)) => output.push_str(&rendered),
            // parsed, but failed to evaluate
            Ok(Err(err)) => output.push_str(&format!("Eval Error: {err}")),
            // failed to parse
            Err(err) => output.push_str(&format!("Parse Error: {err}")),
        }
    }

    output
}

fn eval_stmt(scope: &mut rhai::Scope, stmt: Stmt) -> Result<String, Box<rhai::EvalAltResult>> {
    match stmt {
        Stmt::Literal(lit) => Ok(lit),
        Stmt::Expr(expr) => new_engine()
            .eval_ast_with_scope::<rhai::Dynamic>(scope, &expr)
            .map(|value| value.to_string()),
        Stmt::If(IfChainStmt { ifs, tail }) => {
            for IfStmt { expr, body } in ifs {
                if new_engine().eval_ast_with_scope::<bool>(scope, &expr)? {
                    return Ok(eval(scope, body));
                }
            }

            if let Some(block) = tail {
                return Ok(eval(scope, block));
            }

            Ok(String::new())
        }
    }
}
