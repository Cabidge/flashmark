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

impl<'a> Iterator for Slides<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.string.is_empty() {
            return None;
        }

        let break_points = {
            let mut break_point = 0;
            self.string.split_inclusive("---").map(move |section| {
                break_point += section.len();
                break_point
            })
        };

        for break_point in break_points {
            let (slide, rest) = self.string.split_at(break_point);

            let slide = slide.strip_suffix("---").unwrap_or(slide);

            let Some(slide) = slide
                .is_empty()
                .then_some(slide)
                .or_else(|| slide.strip_suffix('\n'))
            else {
                continue;
            };

            let slide = slide.strip_suffix('\r').unwrap_or(slide);

            let rest = rest.strip_prefix('\r').unwrap_or(rest);

            let Some(rest) = rest
                .is_empty()
                .then_some(rest)
                .or_else(|| rest.strip_prefix('\n'))
            else {
                continue;
            };

            self.string = rest;
            return Some(slide);
        }

        Some(std::mem::replace(&mut self.string, ""))
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