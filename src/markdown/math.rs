use markdown_it::{
    parser::inline::{InlineRule, InlineState},
    MarkdownIt, Node, NodeValue, Renderer,
};

use crate::math::DisplayMode;

pub struct InlineMathRule;

#[derive(Debug)]
pub struct MathNode {
    pub display_mode: DisplayMode,
    pub body: String,
}

pub fn add(md: &mut MarkdownIt) {
    md.inline.add_rule::<InlineMathRule>();
}

const LEFT_DELIM: &str = "$`";
const RIGHT_DELIM: &str = "`$";

impl MathNode {
    pub fn new(display_mode: DisplayMode, input: &str) -> Self {
        use crate::math::{self, parse};

        let mut body = String::new();

        let ast_parser = parse::Parser::new(input);

        math::render_row(ast_parser, &mut body).expect("Writing to string should not fail");

        Self { display_mode, body }
    }
}

impl InlineRule for InlineMathRule {
    const MARKER: char = '$';

    fn run(state: &mut InlineState) -> Option<(Node, usize)> {
        let input = &state.src[state.pos..state.pos_max];

        let input = input.strip_prefix(LEFT_DELIM)?;
        let length = input.find(RIGHT_DELIM)?;

        let node = Node::new(MathNode::new(DisplayMode::Inline, &input[..length]));

        let full_length = LEFT_DELIM.len() + length + RIGHT_DELIM.len();

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
