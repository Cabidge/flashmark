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

    fn try_tokenize_keyword(&mut self) -> Option<token::Keyword> {
        use token::{Function, Grouping, GroupingKind, GroupingSide, Keyword, Symbol};

        let mut symbol_mapping = vec![
            ("+", Symbol::Plus),
            ("-", Symbol::Minus),
            ("/", Symbol::Slash),
            ("*", Symbol::DotProduct),
            ("xx", Symbol::CrossProduct),
            ("^", Symbol::Caret),
            ("_", Symbol::Underscore),
            ("=", Symbol::Equal),
            ("!=", Symbol::NotEqual),
            ("<", Symbol::LessThan),
            (">", Symbol::GreaterThan),
            ("<=", Symbol::LessThanOrEqual),
            (">=", Symbol::GreaterThanOrEqual),
            (":", Symbol::Colon),
            ("in", Symbol::In),
            ("notin", Symbol::NotIn),
            ("sum", Symbol::Sum),
            ("int", Symbol::Integral),
        ];

        let grouping_mapping = [
            ("(", ")", GroupingKind::Paren),
            ("[", "]", GroupingKind::Bracket),
            ("{", "}", GroupingKind::Brace),
        ];

        // add grouping symbols to the symbol mapping
        symbol_mapping.extend(
            grouping_mapping
                .into_iter()
                .flat_map(|(left, right, kind)| {
                    [
                        (left, kind, GroupingSide::Left),
                        (right, kind, GroupingSide::Right),
                    ]
                })
                .map(|(symbol, kind, side)| (symbol, Symbol::Grouping(Grouping { kind, side }))),
        );

        let function_mapping = [
            ("sqrt", Function::Sqrt),
            ("sin", Function::Sin),
            ("cos", Function::Cos),
            ("tan", Function::Tan),
        ];

        let mut keyword_mapping = vec![];

        // add symbol mappings to the keyword mappings
        keyword_mapping.extend(
            symbol_mapping
                .into_iter()
                .map(|(symbol, keyword)| (symbol, Keyword::Symbol(keyword))),
        );

        // add function mappings to the keyword mappings
        keyword_mapping.extend(
            function_mapping
                .into_iter()
                .map(|(symbol, keyword)| (symbol, Keyword::Function(keyword))),
        );

        let mut parser = self.parser.clone();

        // remember the latest keyword match and the parser state
        let mut last_match: Option<(Keyword, StrParser<'a>)> = None;

        // loop until we have no more keywords to match
        while !keyword_mapping.is_empty() {
            let ch = parser.advance()?;

            // remove all keywords that don't match the current character
            keyword_mapping.retain_mut(|(symbol, _)| {
                let Some(rest) = symbol.strip_prefix(ch) else { return false; };
                *symbol = rest;
                true
            });

            if let Some(keyword) = keyword_mapping
                .iter()
                .find_map(|(symbol, keyword)| symbol.is_empty().then_some(*keyword))
            {
                // if we found a keyword, we want to remember the parser state
                last_match = Some((keyword, parser.clone()));
            }
        }

        last_match.map(|(keyword, parser)| {
            self.parser = parser;
            keyword
        })
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = token::Token;

    fn next(&mut self) -> Option<Self::Item> {
        use token::{Literal, Token};

        self.parser.skip_whitespace();

        if self.parser.is_exhausted() {
            return None;
        }

        if self.parser.peek().is_some_and(|ch| ch.is_ascii_digit()) {
            let num_slice = self.parser.consume_while(|ch| ch.is_ascii_digit());
            let num = Box::from(num_slice);

            return Some(Token::Literal(Literal::Number(num)));
        }

        if let Some(keyword) = self.try_tokenize_keyword() {
            return Some(Token::Keyword(keyword));
        }

        if self.parser.consume('\"') {
            todo!("Text literals not implemented yet");
        }

        Some(Token::Literal(Literal::Variable(self.parser.advance()?)))
    }
}
