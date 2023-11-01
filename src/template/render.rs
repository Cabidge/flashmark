use super::{
    parse::{Block, ForBlock, IfChainBlock, Line, Node},
    Environment,
};

pub trait Render {
    fn render(&self, env: &mut Environment, unindent_amount: usize, output: &mut String);
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

impl<'a> Render for Block<'a> {
    fn render(&self, env: &mut Environment, unindent_amount: usize, output: &mut String) {
        let inner_unindent = self.min_indentation().saturating_sub(self.indent);

        let unindent_amount = unindent_amount + inner_unindent;

        for node in self.nodes.iter() {
            node.render(env, unindent_amount, output);
        }
    }
}

impl<'a> Render for IfChainBlock<'a> {
    fn render(&self, env: &mut Environment, unindent_amount: usize, output: &mut String) {
        match self.get_branch(env) {
            Some(Ok(block)) => block.render(env, unindent_amount, output),
            Some(Err(err)) => {
                use std::fmt::Write;

                // the amount of indentation errors should have
                let err_indent = self
                    .min_indentation()
                    .unwrap_or(0)
                    .saturating_sub(unindent_amount);

                writeln!(output, "{:err_indent$}{}", "", err)
                    .expect("writing to string can't fail");
            }
            None => (),
        }
    }
}

impl<'a> Render for ForBlock<'a> {
    fn render(&self, env: &mut Environment, unindent_amount: usize, output: &mut String) {
        use std::fmt::Write;

        // the amount of indentation errors should have
        let err_indent = self.block.indent.saturating_sub(unindent_amount);

        let iterable = match env.eval_ast(&self.iterable) {
            Ok(iterable) => iterable,
            Err(err) => {
                writeln!(output, "{:err_indent$}{}", "", err)
                    .expect("writing to string can't fail");
                return;
            }
        };

        let iterator = match env.get_iter(iterable) {
            Ok(iterator) => iterator,
            Err(value) => {
                writeln!(output, "{:err_indent$}{} is not iterable", "", value)
                    .expect("writing to string can't fail");
                return;
            }
        };

        for item in iterator {
            match item {
                Ok(value) => {
                    env.scope_mut().push(self.binding, value);
                    self.block.render(env, unindent_amount, output);
                    env.scope_mut().pop();
                }
                Err(err) => {
                    writeln!(output, "{:err_indent$}{}", "", err)
                        .expect("writing to string can't fail");
                }
            }
        }
    }
}

impl<'a> Render for Line<'a> {
    fn render(&self, env: &mut Environment, unindent_amount: usize, output: &mut String) {
        let unindented = unindent(self.front, unindent_amount);
        output.push_str(unindented);

        for (expr, text) in &self.expressions {
            use std::fmt::Write;

            let expr = match expr {
                Ok(expr) => expr,
                Err(err) => {
                    write!(output, "{}", err).expect("writing to string can't fail");
                    continue;
                }
            };

            match env.eval_ast::<rhai::Dynamic>(expr) {
                Ok(value) => {
                    write!(output, "{}", value).expect("writing to string can't fail");
                }
                Err(err) => {
                    write!(output, "{}", err).expect("writing to string can't fail");
                }
            }

            output.push_str(text);
        }
        output.push('\n');
    }
}

impl<'a> Render for Node<'a> {
    fn render(&self, env: &mut Environment, unindent_amount: usize, output: &mut String) {
        match self {
            Node::Line(line) => line.render(env, unindent_amount, output),
            Node::If(if_block) => if_block.render(env, unindent_amount, output),
            Node::For(for_block) => for_block.render(env, unindent_amount, output),
            Node::Err { indent, error } => {
                use std::fmt::Write;

                // the amount of indentation errors should have
                let err_indent = indent.saturating_sub(unindent_amount);

                writeln!(output, "{:err_indent$}{}", "", error)
                    .expect("writing to string can't fail");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
