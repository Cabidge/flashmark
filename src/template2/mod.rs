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
        self.nodes.iter().map(Node::indentation).min().unwrap_or(0)
    }
}

impl<'a> Node<'a> {
    fn indentation(&self) -> usize {
        match self {
            Node::Line(line) => {
                let trimmed = line.trim_start();
                if trimmed.is_empty() {
                    0
                } else {
                    line.len() - trimmed.len()
                }
            }
            Node::Block(block) => block.indent,
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

fn parse_block<'a>(
    env: &mut Environment<'_>,
    lines: &mut impl Iterator<Item = &'a str>,
    indent: usize,
) -> Block<'a> {
    let mut rows = vec![];

    while let Some(line) = lines.next() {
        let trimmed = line.trim_start();
        if let Some(directive) = trimmed.strip_prefix('@') {
            // TODO: this only considers space indentation, not tabs
            let indent = line.len() - trimmed.len();

            if let Some(condition) = directive.strip_prefix("if ") {
                let condition = env.engine.compile_expression(condition).unwrap();
                let block = Block {
                    indent,
                    nodes: parse_if(env, condition, lines),
                };

                rows.push(Node::Block(block));

                continue;
            } else if let Some(header) = directive.strip_prefix("for ") {
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

            // otherwise, it's a normal line
        }
    }

    Block {
        indent,
        nodes: rows,
    }
}

fn parse_row<'a>(env: &mut Environment<'_>, line: &'a str) -> Node<'a> {
    todo!()
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
