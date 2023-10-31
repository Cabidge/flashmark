mod directive;

use directive::Directive;

use super::Environment;

pub struct Block<'a> {
    pub indent: usize,
    pub nodes: Vec<Node<'a>>,
}

pub struct IfBlock<'a> {
    pub condition: rhai::AST,
    pub block: Block<'a>,
}

pub struct IfChainBlock<'a> {
    pub if_blocks: Vec<IfBlock<'a>>,
    pub else_block: Option<Block<'a>>,
}

pub struct ForBlock<'a> {
    pub binding: &'a str,
    pub iterable: rhai::AST,
    pub block: Block<'a>,
}

pub struct Line<'a> {
    pub front: &'a str,
    pub expressions: Vec<(rhai::AST, &'a str)>,
}

pub enum Node<'a> {
    Line(Line<'a>),
    If(IfChainBlock<'a>),
    For(ForBlock<'a>),
}

pub fn parse_root<'a>(env: &Environment, lines: &mut impl Iterator<Item = &'a str>) -> Block<'a> {
    parse_block(env, lines, 0, |_| false).0
}

pub fn parse_front_matter(input: &str) -> (Option<&str>, &str) {
    input
        .strip_prefix("---\n")
        .and_then(|stripped| stripped.split_once("---\n"))
        .map(|(front, input)| (Some(front), input))
        .unwrap_or((None, input))
}

fn is_end_directive(directive: &Directive) -> bool {
    directive.name == "end" && directive.args.is_none()
}

fn parse_block<'a>(
    env: &Environment,
    lines: &mut impl Iterator<Item = &'a str>,
    indent: usize,
    mut is_sentinel: impl FnMut(&Directive) -> bool,
) -> (Block<'a>, Option<Directive<'a>>) {
    let mut block = Block {
        indent,
        nodes: vec![],
    };

    while let Some(line) = lines.next() {
        if let Ok(directive) = Directive::try_from(line) {
            if is_sentinel(&directive) {
                return (block, Some(directive));
            }

            if let Some(node) = parse_directive_block(env, directive, lines) {
                block.nodes.push(node);
                continue;
            }
        }

        let line = parse_line(env, line);
        block.nodes.push(Node::Line(line));
    }

    (block, None)
}

fn split_expr_prefix(line: &str) -> Option<(&str, &str)> {
    let mut ignore = false;
    let mut len = 0;
    for ch in line.chars() {
        len += ch.len_utf8();

        if ignore {
            ignore = true;
            continue;
        }

        match ch {
            '\\' => ignore = true,
            '@' => {
                return Some((&line[..(len - 1)], &line[len..]));
            }
            _ => (),
        }
    }

    None
}

fn split_expr(mut line: &str) -> (&str, &str) {
    let mut paren_count = 0;
    while let Some(next) = line.strip_prefix('(') {
        paren_count += 1;
        line = next;
    }

    let mut end = 0;
    let mut streak = 0;
    for ch in line.chars() {
        if paren_count == 0 {
            // if there was no opening paren, we only want to match a single identifier
            if !ch.is_alphabetic() {
                break;
            }
        } else if ch == ')' {
            streak += 1;
        } else if streak < paren_count {
            streak = 0;
        } else {
            break;
        }

        end += ch.len_utf8();
    }

    (&line[..(end - paren_count)], &line[end..])
}

fn parse_line<'a>(env: &Environment, line: &'a str) -> Line<'a> {
    let Some((front, mut rest)) = split_expr_prefix(line) else {
        return Line {
            front: line,
            expressions: vec![],
        };
    };

    let mut expressions = vec![];
    while !rest.is_empty() {
        let (expr, text) = split_expr(rest);
        let expr = env.compile_expr(expr).unwrap();

        let (text, tail) = split_expr_prefix(text).unwrap_or((text, ""));
        rest = tail;

        expressions.push((expr, text));
    }

    Line { front, expressions }
}

fn parse_directive_block<'a>(
    env: &Environment,
    directive: Directive<'a>,
    lines: &mut impl Iterator<Item = &'a str>,
) -> Option<Node<'a>> {
    match (directive.name, directive.args) {
        ("if", Some(condition)) => {
            let block = parse_if_chain(env, condition, lines, directive.indent);

            Some(Node::If(block))
        }
        ("for", Some(header)) => {
            let (binding, iterable) = header.split_once(" in ")?;
            let binding = binding.trim();
            let iterable = env.compile_expr(iterable).unwrap();

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
    env: &Environment,
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
        let condition = env.compile_expr(cond_src).unwrap();

        fn is_sentinel(directive: &Directive<'_>) -> bool {
            matches!(
                (directive.name, directive.args.is_none()),
                ("elif", false) | ("else", true) | ("end", true)
            )
        }

        let (block, closing_directive) = parse_block(env, lines, indent, is_sentinel);

        if_chain.if_blocks.push(IfBlock { condition, block });

        match closing_directive
            .map(|directive| (directive.name, directive.args))
            .unwrap_or(("end", None))
        {
            ("elif", Some(cond)) => cond_src = cond,
            ("else", _) => break,
            _ => return if_chain,
        }
    }

    let (block, _) = parse_block(env, lines, indent, is_end_directive);

    if_chain.else_block = Some(block);

    if_chain
}

impl<'a> Block<'a> {
    pub fn min_indentation(&self) -> usize {
        self.nodes
            .iter()
            .filter_map(Node::indentation)
            .min()
            .unwrap_or(0)
    }
}

impl<'a> IfChainBlock<'a> {
    pub fn min_indentation(&self) -> Option<usize> {
        self.if_blocks
            .iter()
            .map(|if_block| if_block.block.indent)
            .chain(self.else_block.as_ref().map(|block| block.indent))
            .min()
    }

    pub fn get_branch(
        &self,
        env: &mut Environment,
    ) -> Option<Result<&Block<'a>, Box<rhai::EvalAltResult>>> {
        for block in self.if_blocks.iter() {
            match env.eval_ast::<bool>(&block.condition) {
                Ok(true) => return Some(Ok(&block.block)),
                Err(err) => return Some(Err(err)),
                Ok(false) => (),
            }
        }

        self.else_block.as_ref().map(Ok)
    }
}

impl<'a> Line<'a> {
    pub fn indentation(&self) -> Option<usize> {
        let trimmed = self.front.trim_start();

        (!trimmed.is_empty() || !self.expressions.is_empty())
            .then_some(self.front.len() - trimmed.len())
    }
}

impl<'a> Node<'a> {
    pub fn indentation(&self) -> Option<usize> {
        match self {
            Node::Line(line) => line.indentation(),
            Node::If(if_block) => if_block.min_indentation(),
            Node::For(for_block) => Some(for_block.block.indent),
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
}
