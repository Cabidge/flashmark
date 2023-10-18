use std::borrow::Cow;

struct Environment<'a> {
    engine: &'a rhai::Engine,
    scope: &'a mut rhai::Scope<'static>,
}

struct Block<'a> {
    indent: usize,
    nodes: Vec<Node<'a>>,
}

enum Node<'a> {
    Line(Cow<'a, str>),
    Block(Block<'a>),
}

struct Lines<'a> {
    unindent_amount: usize,
    rest: &'a [Node<'a>],
    nested: Option<Box<Lines<'a>>>,
}

struct Directive<'a> {
    indent: usize,
    name: &'a str,
    args: Option<&'a str>,
}

impl<'a> Block<'a> {
    fn lines(&'a self) -> Lines<'a> {
        let unindent_amount = self.min_indentation().saturating_sub(self.indent);

        Lines {
            unindent_amount,
            rest: &self.nodes,
            nested: None,
        }
    }

    fn min_indentation(&self) -> usize {
        self.nodes
            .iter()
            .filter_map(Node::indentation)
            .min()
            .unwrap_or(0)
    }
}

impl<'a> Node<'a> {
    fn indentation(&self) -> Option<usize> {
        match self {
            Node::Line(line) => {
                let trimmed = line.trim_start();
                (!trimmed.is_empty()).then_some(line.len() - trimmed.len())
            }
            Node::Block(block) => Some(block.indent),
        }
    }
}

pub fn render(engine: &rhai::Engine, scope: &mut rhai::Scope<'static>, input: &str) -> String {
    let mut env = Environment { engine, scope };

    let mut output = String::new();
    for line in parse_block(&mut env, &mut input.lines(), 0).lines() {
        output.push_str(line);
        output.push('\n');
    }

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

fn parse_block<'a>(
    env: &mut Environment<'_>,
    lines: &mut impl Iterator<Item = &'a str>,
    indent: usize,
) -> Block<'a> {
    let mut rows = vec![];

    while let Some(line) = lines.next() {
        if let Some(directive) = parse_directive(line) {
            match (directive.name, directive.args) {
                ("if", Some(condition)) => {
                    let condition = env.engine.compile_expression(condition).unwrap();
                    let block = Block {
                        indent,
                        nodes: parse_if(env, condition, lines),
                    };

                    rows.push(Node::Block(block));

                    continue;
                }
                ("for", Some(header)) => {
                    let (binding, iterable) = header.split_once(" in ").unwrap();
                    let binding = binding.trim();
                    let iterable = env.engine.compile_expression(iterable).unwrap();

                    let block = Block {
                        indent,
                        nodes: parse_for(env, binding, iterable, lines),
                    };

                    rows.push(Node::Block(block));

                    continue;
                }
                _ => (),
            }
        }

        let line = parse_line(env, line);
        rows.push(Node::Line(line));
    }

    Block {
        indent,
        nodes: rows,
    }
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

fn parse_if<'a>(
    env: &mut Environment<'_>,
    condition: rhai::AST,
    lines: &mut impl Iterator<Item = &'a str>,
) -> Vec<Node<'a>> {
    todo!()
}

fn parse_for<'a>(
    env: &mut Environment<'_>,
    binding: &str,
    iterable: rhai::AST,
    lines: &mut impl Iterator<Item = &'a str>,
) -> Vec<Node<'a>> {
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

impl<'a> Lines<'a> {
    fn next_indented(&mut self) -> Option<&'a str> {
        if let Some(nested) = &mut self.nested {
            if let Some(line) = nested.next() {
                return Some(line);
            } else {
                self.nested = None;
            }
        }

        let (row, rest) = self.rest.split_first()?;
        self.rest = rest;

        match row {
            Node::Line(line) => Some(line),
            Node::Block(block) => {
                self.nested = Some(Box::new(block.lines()));
                self.next()
            }
        }
    }
}

impl<'a> Iterator for Lines<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_indented()
            .map(|line| unindent(line, self.unindent_amount))
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
