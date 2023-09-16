/// Given a string, return slices of the string separated by a line break,
/// followed by a horizontal rule (---), followed by another line break.
pub struct Slides<'a> {
    string: &'a str,
}

impl<'a> Slides<'a> {
    pub fn new(string: &'a str) -> Self {
        Self { string }
    }
}

fn strip_suffix_newline(input: &str) -> Option<&str> {
    input
        .strip_suffix('\n')
        .map(|input| input.strip_suffix('\r').unwrap_or(input))
}

fn strip_prefix_newline(input: &str) -> Option<&str> {
    input.strip_prefix('\r').unwrap_or(input).strip_prefix('\n')
}

fn empty_or_else<'a>(input: &'a str, f: impl FnOnce() -> Option<&'a str>) -> Option<&'a str> {
    input.is_empty().then_some(input).or_else(f)
}

impl<'a> Iterator for Slides<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.string.is_empty() {
            return None;
        }

        let mut break_points = {
            let mut break_point = 0;
            self.string.split_inclusive("---").map(move |section| {
                break_point += section.len();
                break_point
            })
        };

        let (slide, rest) = break_points
            .find_map(|break_point| {
                let (slide, rest) = self.string.split_at(break_point);

                let slide = slide.strip_suffix("---").unwrap_or(slide);
                let slide = empty_or_else(slide, || strip_suffix_newline(slide))?;
                let rest = empty_or_else(rest, || strip_prefix_newline(rest))?;

                Some((slide, rest))
            })
            .unwrap_or((self.string, ""));

        self.string = rest;
        Some(slide)
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::Slides;

    #[test]
    fn empty() {
        let mut slides = Slides::new("");
        assert_eq!(slides.next(), None);
    }

    #[test]
    fn one_slide() {
        let mut slides = Slides::new("Hello, world!");
        assert_eq!(slides.next(), Some("Hello, world!"));
        assert_eq!(slides.next(), None);
    }

    #[test]
    fn two_slides() {
        let mut slides = Slides::new(
            indoc! {"
                Hello, world!
                ---
                Goodbye, world!
            "}
            .trim_end(),
        );
        assert_eq!(slides.next(), Some("Hello, world!"));
        assert_eq!(slides.next(), Some("Goodbye, world!"));
        assert_eq!(slides.next(), None);
    }

    #[test]
    fn three_slides() {
        let mut slides = Slides::new(
            indoc! {"
                Hello, world!
                ---
                Goodbye, world!
                ---
                Hello again, world!
            "}
            .trim_end(),
        );
        assert_eq!(slides.next(), Some("Hello, world!"));
        assert_eq!(slides.next(), Some("Goodbye, world!"));
        assert_eq!(slides.next(), Some("Hello again, world!"));
        assert_eq!(slides.next(), None);
    }

    #[test]
    fn quad_dash() {
        let mut slides = Slides::new(
            indoc! {"
                Hello, world!
                ----
                Goodbye, world!
            "}
            .trim_end(),
        );
        assert_eq!(slides.next(), Some("Hello, world!\n----\nGoodbye, world!"));
        assert_eq!(slides.next(), None);
    }

    #[test]
    fn same_line_dash() {
        let mut slides = Slides::new(
            indoc! {"
                Hello, world!
                ---Goodbye, world!
            "}
            .trim_end(),
        );
        assert_eq!(slides.next(), Some("Hello, world!\n---Goodbye, world!"));
        assert_eq!(slides.next(), None);
    }

    #[test]
    fn same_line_after_dash() {
        let mut slides = Slides::new(
            indoc! {"
                Hello, world!---
                Goodbye, world!
            "}
            .trim_end(),
        );
        assert_eq!(slides.next(), Some("Hello, world!---\nGoodbye, world!"));
        assert_eq!(slides.next(), None);
    }

    #[test]
    fn no_dash() {
        let mut slides = Slides::new(
            indoc! {"
                Hello, world!
                Goodbye, world!
            "}
            .trim_end(),
        );
        assert_eq!(slides.next(), Some("Hello, world!\nGoodbye, world!"));
        assert_eq!(slides.next(), None);
    }

    #[test]
    fn carriage_return() {
        let mut slides = Slides::new(
            indoc! {"
                Hello, world!\r
                ---\r
                Goodbye, world!
            "}
            .trim_end(),
        );
        assert_eq!(slides.next(), Some("Hello, world!"));
        assert_eq!(slides.next(), Some("Goodbye, world!"));
        assert_eq!(slides.next(), None);
    }

    #[test]
    fn iterate_then_rejoin() {
        let source = indoc! {"
                Hello, world!
                ---
                Goodbye, world!
                ---
                Hello again, world!
            "}
        .trim_end();

        let slides: Vec<_> = Slides::new(source).collect();

        assert_eq!(slides.join("\n---\n"), source);
    }

    #[test]
    fn empty_slide() {
        let mut slides = Slides::new(
            indoc! {"
                Hello, world!
                ---
                ---
                Goodbye, world!
            "}
            .trim_end(),
        );
        assert_eq!(slides.next(), Some("Hello, world!"));
        assert_eq!(slides.next(), Some(""));
        assert_eq!(slides.next(), Some("Goodbye, world!"));
        assert_eq!(slides.next(), None);
    }

    #[test]
    fn empty_last_slide() {
        let mut slides = Slides::new(
            indoc! {"
                Hello, world!
                ---
                Goodbye, world!
                ---
            "}
            .trim_end(),
        );
        assert_eq!(slides.next(), Some("Hello, world!"));
        assert_eq!(slides.next(), Some("Goodbye, world!"));
        assert_eq!(slides.next(), None);
    }

    #[test]
    fn empty_first_slide() {
        let mut slides = Slides::new(
            indoc! {"
                ---
                Hello, world!
                ---
                Goodbye, world!
            "}
            .trim_end(),
        );
        assert_eq!(slides.next(), Some(""));
        assert_eq!(slides.next(), Some("Hello, world!"));
        assert_eq!(slides.next(), Some("Goodbye, world!"));
        assert_eq!(slides.next(), None);
    }

    #[test]
    fn complex() {
        let mut slides = Slides::new(
            indoc! {"
                Hello, world!
                ---
                Goodbye, world!
                ---
                ---
                Hello again, world!
                ---
                ---
                ---
            "}
            .trim_end(),
        );
        assert_eq!(slides.next(), Some("Hello, world!"));
        assert_eq!(slides.next(), Some("Goodbye, world!"));
        assert_eq!(slides.next(), Some(""));
        assert_eq!(slides.next(), Some("Hello again, world!"));
        assert_eq!(slides.next(), Some(""));
        assert_eq!(slides.next(), Some(""));
        assert_eq!(slides.next(), None);
    }
}
