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
        let Some(block) = self.get_branch(env) else {
            return;
        };

        block.render(env, unindent_amount, output);
    }
}

impl<'a> Render for ForBlock<'a> {
    fn render(&self, env: &mut Environment, unindent_amount: usize, output: &mut String) {
        let iterable = env.eval_ast(&self.iterable).unwrap();
        let iterator = env.get_iter(iterable).unwrap();

        for item in iterator {
            env.scope_mut().push(self.binding, item.unwrap());

            self.block.render(env, unindent_amount, output);

            env.scope_mut().pop();
        }
    }
}

impl<'a> Render for Line<'a> {
    fn render(&self, env: &mut Environment, unindent_amount: usize, output: &mut String) {
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

impl<'a> Render for Node<'a> {
    fn render(&self, env: &mut Environment, unindent_amount: usize, output: &mut String) {
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
