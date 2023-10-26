pub mod markdown;
pub mod math;
pub mod parsing;
pub mod slides;
pub mod template;

pub fn render(input: &str) -> Vec<String> {
    use markdown_it::MarkdownIt;

    let mut md = MarkdownIt::new();

    markdown_it::plugins::cmark::add(&mut md);
    markdown::math::add(&mut md);

    slides::Slides::new(&template::render(input))
        .map(|slide| md.parse(slide).render())
        .collect()
}
