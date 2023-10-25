pub mod environment;

pub use environment::Environment;

struct Block<'a> {
    indent: usize,
    nodes: Vec<Node<'a>>,
}

struct IfBlock<'a> {
    condition: rhai::AST,
    block: Block<'a>,
}

struct IfChainBlock<'a> {
    if_blocks: Vec<IfBlock<'a>>,
    else_block: Option<Block<'a>>,
}

struct ForBlock<'a> {
    binding: &'a str,
    iterable: rhai::AST,
    block: Block<'a>,
}

struct Line<'a> {
    front: &'a str,
    expressions: Vec<(rhai::AST, &'a str)>,
}

enum Node<'a> {
    Line(Line<'a>),
    If(IfChainBlock<'a>),
    For(ForBlock<'a>),
}

struct Directive<'a> {
    indent: usize,
    name: &'a str,
    args: Option<&'a str>,
}

pub fn render(engine: &rhai::Engine, scope: &mut rhai::Scope<'static>, input: &str) -> String {
    let mut env = Environment::new(engine, scope);

    let mut output = String::new();
    parse_root(&mut env, &mut input.lines()).render(&mut env, 0, &mut output);

    output
}

fn parse_directive(line: &str) -> Option<Directive<'_>> {
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix('@')?;

    let indent = line.len() - trimmed.len();

    let Some((name, args)) = rest.split_once(' ') else {
        return Some(Directive {
            indent,
            name: rest,
            args: None,
        });
    };

    let args = Some(args.trim());

    Some(Directive { indent, name, args })
}

fn parse_root<'a>(
    env: &mut Environment<'_>,
    lines: &mut impl Iterator<Item = &'a str>,
) -> Block<'a> {
    parse_block(env, lines, 0, |_| false).0
}

fn is_end_directive(directive: &Directive<'_>) -> bool {
    directive.name == "end" && directive.args.is_none()
}

fn parse_block<'a>(
    env: &mut Environment<'_>,
    lines: &mut impl Iterator<Item = &'a str>,
    indent: usize,
    mut is_sentinel: impl FnMut(&Directive<'a>) -> bool,
) -> (Block<'a>, Option<Directive<'a>>) {
    let mut rows = vec![];

    let mut closing_directive = None;

    while let Some(line) = lines.next() {
        if let Some(directive) = parse_directive(line) {
            if is_sentinel(&directive) {
                closing_directive = Some(directive);
                break;
            }

            if let Some(node) = parse_directive_block(env, directive, lines) {
                rows.push(node);
                continue;
            }
        }

        let line = parse_line(env, line);
        rows.push(Node::Line(line));
    }

    let block = Block {
        indent,
        nodes: rows,
    };

    (block, closing_directive)
}

fn parse_line<'a>(env: &mut Environment<'_>, line: &'a str) -> Line<'a> {
    let Some((front, mut rest)) = line.split_once("@(") else {
        return Line {
            front: line,
            expressions: vec![],
        };
    };

    fn split_expression(s: &str) -> (&str, &str) {
        s.split_once(')').unwrap_or((s, ""))
    }

    let mut expressions = vec![];
    while !rest.is_empty() {
        let (expr, text) = split_expression(rest);
        let expr = env.engine().compile_expression(expr).unwrap();

        let (text, tail) = text.split_once("@(").unwrap_or((text, ""));
        rest = tail;

        expressions.push((expr, text));
    }

    Line { front, expressions }
}

fn parse_directive_block<'a>(
    env: &mut Environment,
    directive: Directive<'a>,
    lines: &mut impl Iterator<Item = &'a str>,
) -> Option<Node<'a>> {
    match (directive.name, directive.args) {
        ("if", Some(condition)) => {
            let block = parse_if_chain(env, condition, lines, directive.indent);

            Some(Node::If(block))
        }
        ("for", Some(header)) => {
            let (binding, iterable) = header.split_once(" in ").unwrap();
            let binding = binding.trim();
            let iterable = env.engine().compile_expression(iterable).unwrap();

            let (block, _) = parse_block(env, lines, directive.indent, is_end_directive);

            Some(Node::For(ForBlock {
                binding,
                iterable,
                block,
            }))
        }
        _ => None,
    }
}

fn parse_if_chain<'a>(
    env: &mut Environment<'_>,
    condition: &str,
    lines: &mut impl Iterator<Item = &'a str>,
    indent: usize,
) -> IfChainBlock<'a> {
    let mut if_chain = IfChainBlock {
        if_blocks: vec![],
        else_block: None,
    };

    let mut cond_src = condition;
    loop {
        let condition = env.engine().compile_expression(cond_src).unwrap();

        fn is_sentinel(directive: &Directive<'_>) -> bool {
            matches!(
                (directive.name, directive.args.is_none()),
                ("elif", false) | ("else", true) | ("end", true)
            )
        }

        let (block, closing_directive) = parse_block(env, lines, indent, is_sentinel);

        if_chain.if_blocks.push(IfBlock { condition, block });

        let Some(closing_directive) = closing_directive else {
            return if_chain;
        };

        match (closing_directive.name, closing_directive.args) {
            ("elif", Some(cond)) => cond_src = cond,
            ("else", _) => break,
            _ => return if_chain,
        }
    }

    let (block, _) = parse_block(env, lines, indent, is_end_directive);

    if_chain.else_block = Some(block);

    if_chain
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
    fn min_indentation(&self) -> usize {
        self.nodes
            .iter()
            .filter_map(Node::indentation)
            .min()
            .unwrap_or(0)
    }

    fn render(&self, env: &mut Environment<'_>, unindent_amount: usize, output: &mut String) {
        let inner_unindent = self.min_indentation().saturating_sub(self.indent);

        let unindent_amount = unindent_amount + inner_unindent;

        for node in self.nodes.iter() {
            node.render(env, unindent_amount, output);
        }
    }
}

impl<'a> IfChainBlock<'a> {
    fn min_indentation(&self) -> Option<usize> {
        self.if_blocks
            .iter()
            .map(|if_block| if_block.block.indent)
            .chain(self.else_block.as_ref().map(|block| block.indent))
            .min()
    }

    fn get_branch(&self, env: &mut Environment<'_>) -> Option<&Block<'a>> {
        for block in self.if_blocks.iter() {
            if env.eval_ast::<bool>(&block.condition).unwrap() {
                return Some(&block.block);
            }
        }

        self.else_block.as_ref()
    }

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
    fn indentation(&self) -> Option<usize> {
        let trimmed = self.front.trim_start();

        (!trimmed.is_empty() || !self.expressions.is_empty())
            .then_some(self.front.len() - trimmed.len())
    }

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
    fn indentation(&self) -> Option<usize> {
        match self {
            Node::Line(line) => line.indentation(),
            Node::If(if_block) => if_block.min_indentation(),
            Node::For(for_block) => Some(for_block.block.indent),
        }
    }

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
