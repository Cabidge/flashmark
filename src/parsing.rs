#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StrParser<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> StrParser<'a> {
    /// Create a new `StrParser` from a string slice.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// use flashmark::parsing::StrParser;
    ///
    /// let parser = StrParser::new("Hello, World!");
    /// ```
    pub fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    /// Returns the current position of the parser.
    ///
    /// # Examples
    /// ```
    /// use flashmark::parsing::StrParser;
    ///
    /// let mut parser = StrParser::new("Hello, World!");
    /// assert_eq!(parser.position(), 0);
    ///
    /// // advance the parser.
    /// assert_eq!(parser.advance(), Some('H'));
    /// assert_eq!(parser.advance(), Some('e'));
    /// assert_eq!(parser.position(), 2);
    /// ```
    pub fn position(&self) -> usize {
        self.position
    }

    /// Returns the input.
    ///
    /// # Examples
    /// ```
    /// use flashmark::parsing::StrParser;
    ///
    /// let mut parser = StrParser::new("Hello, World!");
    /// assert_eq!(parser.input(), "Hello, World!");
    ///
    /// // advancing the parser does not change the input.
    /// assert_eq!(parser.advance(), Some('H'));
    /// assert_eq!(parser.advance(), Some('e'));
    /// assert_eq!(parser.input(), "Hello, World!");
    /// ```
    pub fn input(&self) -> &str {
        self.input
    }

    /// Tries to rewind the parser to a given position.
    /// Fails if the byte at the given position is not a valid char boundary.
    ///
    /// Returns true if it was successful.
    ///
    /// # Examples
    /// ```
    /// use flashmark::parsing::StrParser;
    ///
    /// let mut parser = StrParser::new("Hello, World!");
    /// assert_eq!(parser.position(), 0);
    ///
    /// // advance the parser.
    /// assert_eq!(parser.advance(), Some('H'));
    /// assert_eq!(parser.advance(), Some('e'));
    /// assert_eq!(parser.advance(), Some('l'));
    /// assert_eq!(parser.advance(), Some('l'));
    ///
    /// // rewind the parser.
    /// assert!(parser.try_rewind(1));
    /// assert_eq!(parser.position(), 1);
    /// assert_eq!(parser.advance(), Some('e'));
    /// assert_eq!(parser.advance(), Some('l'));
    /// assert_eq!(parser.advance(), Some('l'));
    /// ```
    pub fn try_rewind(&mut self, position: usize) -> bool {
        if self.input.is_char_boundary(position) {
            self.position = position;
            true
        } else {
            false
        }
    }

    /// Returns if the parser reached the end of the input.
    ///
    /// # Examples
    /// ```
    /// use flashmark::parsing::StrParser;
    ///
    /// let mut parser = StrParser::new("Hello, World!");
    /// assert!(!parser.is_exhausted());
    ///
    /// assert_eq!(parser.consume_rest(), "Hello, World!");
    /// assert!(parser.is_exhausted());
    /// ```
    pub fn is_exhausted(&self) -> bool {
        self.position >= self.input.len()
    }

    /// Returns the remaining input.
    ///
    /// # Examples
    /// ```
    /// use flashmark::parsing::StrParser;
    ///
    /// let mut parser = StrParser::new("Hello, World!");
    /// assert_eq!(parser.rest(), "Hello, World!");
    ///
    /// assert_eq!(parser.advance(), Some('H'));
    /// assert_eq!(parser.rest(), "ello, World!");
    /// ```
    pub fn rest(&self) -> &str {
        &self.input[self.position..]
    }

    /// Skip all of the whitespace.
    ///
    /// # Examples
    /// ```
    /// use flashmark::parsing::StrParser;
    ///
    /// let mut parser = StrParser::new("  \t\nHello, World!");
    /// parser.skip_whitespace();
    /// assert_eq!(parser.rest(), "Hello, World!");
    /// ```
    pub fn skip_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }

    /// Get the next character without advancing the parser.
    ///
    /// # Examples
    /// ```
    /// use flashmark::parsing::StrParser;
    ///
    /// let mut parser = StrParser::new("Hello, World!");
    /// assert_eq!(parser.peek(), Some('H'));
    /// assert_eq!(parser.peek(), Some('H'));
    ///
    /// assert_eq!(parser.advance(), Some('H'));
    /// assert_eq!(parser.peek(), Some('e'));
    /// assert_eq!(parser.rest(), "ello, World!");
    /// ```
    pub fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    /// Tries to advance the parser by one character.
    ///
    /// Returns the character if it exists.
    ///
    /// # Examples
    /// ```
    /// use flashmark::parsing::StrParser;
    ///
    /// let mut parser = StrParser::new("Hello, World!");
    /// assert_eq!(parser.advance(), Some('H'));
    /// assert_eq!(parser.advance(), Some('e'));
    /// assert_eq!(parser.advance(), Some('l'));
    /// assert_eq!(parser.advance(), Some('l'));
    /// assert_eq!(parser.advance(), Some('o'));
    /// assert_eq!(parser.rest(), ", World!");
    /// ```
    pub fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.position += ch.len_utf8();
        Some(ch)
    }

    /// Tries to consume a specific character.
    ///
    /// Returns if the character was consumed.
    ///
    /// # Examples
    /// ```
    /// use flashmark::parsing::StrParser;
    ///
    /// let mut parser = StrParser::new("Hello, World!");
    /// assert!(parser.consume('H'));
    /// assert!(!parser.consume('H'));
    /// assert_eq!(parser.rest(), "ello, World!");
    /// ```
    pub fn consume(&mut self, ch: char) -> bool {
        if self.rest().starts_with(ch) {
            self.position += ch.len_utf8();
            true
        } else {
            false
        }
    }

    /// Tries to consume a character that satisfies a given predicate.
    ///
    /// Returns the consumed character.
    ///
    /// # Examples
    /// ```
    /// use flashmark::parsing::StrParser;
    ///
    /// let mut parser = StrParser::new("123abc");
    /// assert_eq!(parser.consume_if(char::is_numeric), Some('1'));
    /// assert_eq!(parser.consume_if(char::is_numeric), Some('2'));
    /// assert_eq!(parser.consume_if(char::is_numeric), Some('3'));
    /// assert_eq!(parser.consume_if(char::is_numeric), None);
    /// assert_eq!(parser.rest(), "abc");
    /// ```
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
    ///
    /// Returns if the string was consumed.
    ///
    /// # Examples
    /// ```
    /// use flashmark::parsing::StrParser;
    ///
    /// let mut parser = StrParser::new("Hello, World!");
    /// assert!(parser.consume_str("Hello"));
    /// assert!(!parser.consume_str("Hello"));
    /// assert_eq!(parser.rest(), ", World!");
    /// ```
    pub fn consume_str(&mut self, s: &str) -> bool {
        if self.rest().starts_with(s) {
            self.position += s.len();
            true
        } else {
            false
        }
    }

    /// Tries to consume the input while a given predicate is true.
    ///
    /// Returns the consumed input.
    ///
    /// # Example
    /// ```
    /// use flashmark::parsing::StrParser;
    ///
    /// let mut parser = StrParser::new("123abc");
    /// assert_eq!(parser.consume_while(char::is_numeric), "123");
    /// assert_eq!(parser.rest(), "abc");
    /// ```
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
    ///
    /// Returns the consumed input.
    ///
    /// # Example
    /// ```
    /// use flashmark::parsing::StrParser;
    ///
    /// let mut parser = StrParser::new("Hello, World!");
    /// assert_eq!(parser.consume_rest(), "Hello, World!");
    /// ```
    pub fn consume_rest(&mut self) -> &str {
        let start = std::mem::replace(&mut self.position, self.input.len());
        &self.input[start..]
    }
}
