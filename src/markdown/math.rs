use markdown_it::{
    parser::inline::{InlineRule, InlineState},
    MarkdownIt, Node, NodeValue, Renderer,
};

use crate::math::DisplayMode;

use super::{DollarTickDelimiter, InlineDelimiter};

pub struct InlineMathRule<D: InlineDelimiter = DollarTickDelimiter>(std::marker::PhantomData<D>);

#[derive(Debug)]
pub struct MathNode {
    pub display_mode: DisplayMode,
    pub body: String,
}

pub fn add(md: &mut MarkdownIt) {
    md.inline.add_rule::<InlineMathRule>();
}

impl MathNode {
    pub fn new(display_mode: DisplayMode, input: &str) -> Self {
        use crate::math::{self, parse};

        let mut body = String::new();

        let ast_parser = parse::Parser::new(input);

        math::render_row(ast_parser, &mut body).expect("Writing to string should not fail");

        Self { display_mode, body }
    }
}

impl<D: InlineDelimiter> InlineRule for InlineMathRule<D> {
    const MARKER: char = D::MARKER;

    fn run(state: &mut InlineState) -> Option<(Node, usize)> {
        let input = &state.src[state.pos..state.pos_max];

        let input = input.strip_prefix(D::LEFT_DELIM)?;
        let length = input.find(D::RIGHT_DELIM)?;

        let node = Node::new(MathNode::new(DisplayMode::Inline, &input[..length]));

        let full_length = D::LEFT_DELIM.len() + length + D::RIGHT_DELIM.len();

        Some((node, full_length))
    }
}

impl NodeValue for MathNode {
    fn render(&self, _node: &Node, fmt: &mut dyn Renderer) {
        let display = match self.display_mode {
            DisplayMode::Inline => "inline",
            DisplayMode::Block => "block",
        };

        fmt.open("math", &[("display", display.to_string())]);
        fmt.text_raw(&self.body);
        fmt.close("math");
    }
}
