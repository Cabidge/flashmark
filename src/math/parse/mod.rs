pub mod expressions;
pub mod tokenize;

pub struct Parser<'a> {
    token_stream: tokenize::Tokenizer<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            token_stream: tokenize::Tokenizer::new(input),
        }
    }
}
