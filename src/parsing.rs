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

    /// Get the next character without advancing the parser.
    pub fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    /// Tries to advance the parser by one character.
    /// Returns the character if it exists.
    pub fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
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

    /// Tries to consume a character that satisfies a given predicate.
    /// Returns the consumed character.
    pub fn consume_if(&mut self, mut predicate: impl FnMut(char) -> bool) -> Option<char> {
        match self.peek() {
            Some(ch) if predicate(ch) => {
                self.position += ch.len_utf8();
                Some(ch)
            }
            _ => None,
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
        let new_position = self
            .rest()
            .char_indices()
            .find_map(|(i, ch)| (!predicate(ch)).then_some(self.position + i))
            .unwrap_or(self.input.len());

        let start = std::mem::replace(&mut self.position, new_position);
        &self.input[start..self.position]
    }

    /// Consumes the rest of the input.
    pub fn consume_rest(&mut self) -> &str {
        let start = std::mem::replace(&mut self.position, self.input.len());
        &self.input[start..]
    }
}
