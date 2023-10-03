use markdown_it::{
    parser::inline::{InlineRule, InlineState},
    Node,
};

pub struct InlineMathRule;

impl InlineRule for InlineMathRule {
    const MARKER: char = '$';

    fn run(state: &mut InlineState) -> Option<(Node, usize)> {
        None
    }
}
