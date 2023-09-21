use crate::parsing::StrParser;

pub mod token;

pub struct Tokenizer<'a> {
    parser: StrParser<'a>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            parser: StrParser::new(input),
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = token::Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let _ = self.parser; // remove warnings
        todo!()
    }
}
