const INPUT: &str = "
Hello $`2 in { 1, 2, 3 }`$ world!

```math
2 in { 1, 2, 3 }
```
";

fn main() {
    let mut md = markdown_it::MarkdownIt::new();

    markdown_it::plugins::cmark::add(&mut md);
    flashmark::markdown::math::add(&mut md);

    let html = md.parse(INPUT).render();

    println!("{}", html);
}
