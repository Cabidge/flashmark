use markdown_it::{
    parser::core::CoreRule, plugins::cmark::block::fence::CodeFence, MarkdownIt, Node,
};

use super::MathNode;

pub struct MathFenceRule;

const LANGUAGE: &str = "math";

impl CoreRule for MathFenceRule {
    fn run(root: &mut Node, _md: &MarkdownIt) {
        root.walk_mut(|node, _depth| {
            let Some(code_block) = node.cast_mut::<CodeFence>() else {
                return;
            };

            if code_block.info.trim() != LANGUAGE {
                return;
            }

            let mut math_node = Node::new(MathNode::new(&code_block.content));
            math_node.attrs.push(("display", "block".into()));

            *node = math_node;
        });
    }
}
