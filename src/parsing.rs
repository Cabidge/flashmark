#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct StrParser<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> StrParser<'a> {
    /// Create a new `StrParser` from a string slice.
    pub fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    /// Returns the current position of the parser.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Returns the input.
    pub fn input(&self) -> &str {
        self.input
    }

    /// Tries to rewind the parser to a given position.
    /// Fails if the byte at the given position is not a valid char boundary.
    /// Returns true if it was successful.
    pub fn try_rewind(&mut self, position: usize) -> bool {
        if self.input.is_char_boundary(position) {
            self.position = position;
            true
        } else {
            false
        }
    }

    /// Returns if the parser reached the end of the input.
    pub fn is_exhausted(&self) -> bool {
        self.position >= self.input.len()
    }

    /// Returns the remaining input.
    pub fn rest(&self) -> &str {
        &self.input[self.position..]
    }

    /// Skip all of the whitespace.
    pub fn skip_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }

    /// Tries to advance the parser by one character.
    /// Returns the character if it exists.
    pub fn next_char(&mut self) -> Option<char> {
        let ch = self.rest().chars().next()?;
        self.position += ch.len_utf8();
        Some(ch)
    }

    /// Tries to consume a specific character.
    /// Returns if the character was consumed.
    pub fn consume(&mut self, ch: char) -> bool {
        if self.rest().starts_with(ch) {
            self.position += ch.len_utf8();
            true
        } else {
            false
        }
    }

    /// Tries to consume a specific string.
    /// Returns if the string was consumed.
    pub fn consume_str(&mut self, s: &str) -> bool {
        if self.rest().starts_with(s) {
            self.position += s.len();
            true
        } else {
            false
        }
    }

    /// Tries to consume the input while a given predicate is true.
    /// Returns the consumed input.
    pub fn consume_while(&mut self, mut predicate: impl FnMut(char) -> bool) -> &str {
        let Some(length) = self
            .rest()
            .char_indices()
            .find_map(|(i, ch)| (!predicate(ch)).then_some(i))
        else {
            return self.consume_rest();
        };

        let start = self.position;
        self.position += length;

        &self.input[start..self.position]
    }

    /// Consumes the rest of the input.
    pub fn consume_rest(&mut self) -> &str {
        let start = std::mem::replace(&mut self.position, self.input.len());
        &self.input[start..]
    }
}
