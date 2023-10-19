use std::borrow::Cow;

struct Environment<'a> {
    engine: &'a rhai::Engine,
    scope: &'a mut rhai::Scope<'static>,
}

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

enum Node<'a> {
    Line(Cow<'a, str>),
    If(IfChainBlock<'a>),
    For(ForBlock<'a>),
}

struct Directive<'a> {
    indent: usize,
    name: &'a str,
    args: Option<&'a str>,
}

pub fn render(engine: &rhai::Engine, scope: &mut rhai::Scope<'static>, input: &str) -> String {
    let mut env = Environment { engine, scope };

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

            match (directive.name, directive.args) {
                ("if", Some(condition)) => {
                    let block = parse_if_chain(env, condition, lines, directive.indent);

                    rows.push(Node::If(block));

                    continue;
                }
                ("for", Some(header)) => {
                    let (binding, iterable) = header.split_once(" in ").unwrap();
                    let binding = binding.trim();
                    let iterable = env.engine.compile_expression(iterable).unwrap();

                    let (block, _) = parse_block(env, lines, directive.indent, |directive| {
                        directive.name == "end" && directive.args.is_none()
                    });

                    rows.push(Node::For(ForBlock {
                        binding,
                        iterable,
                        block,
                    }));

                    continue;
                }
                _ => (),
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

fn parse_line<'a>(env: &mut Environment<'_>, line: &'a str) -> Cow<'a, str> {
    if !line.contains("@(") {
        return Cow::Borrowed(line);
    }

    let mut output = String::new();
    let mut line = line;
    while let Some((head, rest)) = line.split_once("@(") {
        use std::fmt::Write;

        let (expr, rest) = rest.split_once(')').unwrap_or((rest, ""));
        line = rest;

        let value = env
            .engine
            .eval_expression_with_scope::<rhai::Dynamic>(env.scope, expr)
            .unwrap();

        write!(output, "{}{}", head, value).expect("Writing to String should never fail");
    }
    output.push_str(line);

    Cow::Owned(output)
}

fn parse_if_chain<'a>(
    env: &mut Environment<'_>,
    condition: &str,
    lines: &mut impl Iterator<Item = &'a str>,
    indent: usize,
) -> IfChainBlock<'a> {
    todo!()
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
            if env.engine.eval_ast::<bool>(&block.condition).unwrap() {
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
        let iterable = env.engine.eval_ast::<rhai::Array>(&self.iterable).unwrap();

        for item in iterable.iter() {
            env.scope.push(self.binding, item.clone());

            self.block.render(env, unindent_amount, output);

            env.scope.pop();
        }
    }
}

impl<'a> Node<'a> {
    fn indentation(&self) -> Option<usize> {
        match self {
            Node::Line(line) => {
                let trimmed = line.trim_start();
                (!trimmed.is_empty()).then_some(line.len() - trimmed.len())
            }
            Node::If(if_block) => if_block.min_indentation(),
            Node::For(for_block) => Some(for_block.block.indent),
        }
    }

    fn render(&self, env: &mut Environment<'_>, unindent_amount: usize, output: &mut String) {
        match self {
            Node::Line(line) => {
                output.push_str(unindent(line, unindent_amount));
                output.push('\n');
            }
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
        use std::borrow::Cow;

        #[test]
        fn line() {
            let node = Node::Line(Cow::Borrowed("  hello"));
            assert_eq!(node.indentation(), Some(2));
        }

        #[test]
        fn empty_line() {
            let node = Node::Line(Cow::Borrowed("  "));
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
