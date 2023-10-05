use markdown_it::{
    parser::{core::CoreRule, extset::MarkdownItExt},
    plugins::cmark::block::fence::CodeFence,
    MarkdownIt, Node,
};

use super::MathNode;

pub struct MathFenceRule;

#[derive(Debug, Clone, Copy)]
pub struct MathFenceLanguage(&'static str);

impl CoreRule for MathFenceRule {
    fn run(root: &mut Node, md: &MarkdownIt) {
        let MathFenceLanguage(language) = md.ext.get().copied().unwrap_or_default();

        root.walk_mut(|node, _depth| {
            let Some(code_block) = node.cast_mut::<CodeFence>() else {
                return;
            };

            if code_block.info.trim() != language {
                return;
            }

            let mut math_node = Node::new(MathNode::new_block(&code_block.content));
            math_node.attrs.push(("display", "block".into()));

            *node = math_node;
        });
    }
}

impl MarkdownItExt for MathFenceLanguage {}

impl Default for MathFenceLanguage {
    fn default() -> Self {
        Self("math")
    }
}
