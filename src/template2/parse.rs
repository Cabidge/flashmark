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

struct Directive<'a> {
    indent: usize,
    name: &'a str,
    args: Option<&'a str>,
}

pub fn parse_root<'a>(
    env: &mut Environment<'_>,
    lines: &mut impl Iterator<Item = &'a str>,
) -> Block<'a> {
    parse_block(env, lines, 0, |_| false).0
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
