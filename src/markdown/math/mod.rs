pub mod block;
pub mod inline;

pub use self::block::MathCodeBlockCoreRule;
pub use self::inline::InlineMathRule;

use markdown_it::{MarkdownIt, Node, NodeValue, Renderer};

#[derive(Debug)]
pub struct MathNode {
    pub body: String,
}

pub fn add(md: &mut MarkdownIt) {
    md.add_rule::<MathCodeBlockCoreRule>();
    md.inline.add_rule::<InlineMathRule>();
}

impl MathNode {
    pub fn new(input: &str) -> Self {
        use crate::math::{self, parse};

        let mut body = String::new();

        let ast_parser = parse::Parser::new(input);

        math::render_row(ast_parser, &mut body).expect("Writing to string should not fail");

        Self { body }
    }
}

impl NodeValue for MathNode {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        fmt.open("math", &node.attrs);
        fmt.text_raw(&self.body);
        fmt.close("math");
    }
}
