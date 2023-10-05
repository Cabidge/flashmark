use std::marker::PhantomData;

use markdown_it::{
    parser::inline::{InlineRule, InlineState},
    Node,
};

use crate::markdown::{DollarTickDelimiter, InlineDelimiter};

use super::MathNode;

type DefaultDelimiter = DollarTickDelimiter;

pub struct InlineMathRule<D: InlineDelimiter = DefaultDelimiter>(PhantomData<D>);

impl<D: InlineDelimiter> InlineRule for InlineMathRule<D> {
    const MARKER: char = D::MARKER;

    fn run(state: &mut InlineState) -> Option<(Node, usize)> {
        let input = &state.src[state.pos..state.pos_max];

        let input = input.strip_prefix(D::LEFT_DELIM)?;
        let length = input.find(D::RIGHT_DELIM)?;

        let mut node = Node::new(MathNode::new(&input[..length]));
        node.attrs.push(("display", "inline".into()));

        let full_length = D::LEFT_DELIM.len() + length + D::RIGHT_DELIM.len();

        Some((node, full_length))
    }
}
