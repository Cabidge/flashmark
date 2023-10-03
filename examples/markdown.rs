fn main() {
    let mut md = markdown_it::MarkdownIt::new();

    markdown_it::plugins::cmark::add(&mut md);

    md.inline
        .add_rule::<flashmark::markdown::math::InlineMathRule>();

    let html = md.parse("Hello $`2 in { 1, 2, 3 }`$ world!").render();

    println!("{}", html);
}
