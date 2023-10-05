pub mod inline;

pub use self::inline::InlineMathRule;

use markdown_it::{MarkdownIt, Node, NodeValue, Renderer};

use crate::math::DisplayMode;

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
