use markdown_it::{
    parser::inline::{InlineRule, InlineState},
    Node, NodeValue, Renderer, MarkdownIt,
};

pub struct InlineMathRule;

#[derive(Debug)]
pub struct InlineMathNode {
    pub rendered_body: String,
}

pub fn add(md: &mut MarkdownIt) {
    md.inline.add_rule::<InlineMathRule>();
}

const LEFT_DELIM: &str = "$`";
const RIGHT_DELIM: &str = "`$";

impl InlineRule for InlineMathRule {
    const MARKER: char = '$';

    fn run(state: &mut InlineState) -> Option<(Node, usize)> {
        let input = &state.src[state.pos..state.pos_max];

        let input = input.strip_prefix(LEFT_DELIM)?;
        let length = input.find(RIGHT_DELIM)?;

        let rendered_body = crate::math::render(&input[..length], crate::math::DisplayMode::Inline);
        let node = Node::new(InlineMathNode { rendered_body });

        let full_length = LEFT_DELIM.len() + length + RIGHT_DELIM.len();

        Some((node, full_length))
    }
}

impl NodeValue for InlineMathNode {
    fn render(&self, _node: &Node, fmt: &mut dyn Renderer) {
        fmt.text_raw(&self.rendered_body);
    }
}
