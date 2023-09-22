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

    const SIMPLE_SYMBOL_MAPPING: [(&str, token::Symbol); 18] = [
        ("+", token::Symbol::Plus),
        ("-", token::Symbol::Minus),
        ("/", token::Symbol::Slash),
        ("*", token::Symbol::DotProduct),
        ("xx", token::Symbol::CrossProduct),
        ("^", token::Symbol::Caret),
        ("_", token::Symbol::Underscore),
        ("=", token::Symbol::Equal),
        ("!=", token::Symbol::NotEqual),
        ("<", token::Symbol::LessThan),
        (">", token::Symbol::GreaterThan),
        ("<=", token::Symbol::LessThanOrEqual),
        (">=", token::Symbol::GreaterThanOrEqual),
        (":", token::Symbol::Colon),
        ("in", token::Symbol::In),
        ("notin", token::Symbol::NotIn),
        ("sum", token::Symbol::Sum),
        ("int", token::Symbol::Integral),
    ];

    const GROUPING_MAPPING: [(&str, &str, token::GroupingKind); 3] = [
        ("(", ")", token::GroupingKind::Paren),
        ("[", "]", token::GroupingKind::Bracket),
        ("{", "}", token::GroupingKind::Brace),
    ];

    const FUNCTION_MAPPING: [(&str, token::Function); 4] = [
        ("sqrt", token::Function::Sqrt),
        ("sin", token::Function::Sin),
        ("cos", token::Function::Cos),
        ("tan", token::Function::Tan),
    ];

    fn try_tokenize_keyword(&mut self) -> Option<token::Keyword> {
        use token::{Grouping, GroupingSide, Keyword, Symbol};

        let mut keyword_mapping = vec![];

        // add symbol mappings to the keyword mappings
        keyword_mapping.extend(
            Self::SIMPLE_SYMBOL_MAPPING
                .into_iter()
                .map(|(symbol, keyword)| (symbol, Keyword::Symbol(keyword))),
        );

        // add grouping mappings to the keyword mappings
        keyword_mapping.extend(
            Self::GROUPING_MAPPING
                .into_iter()
                .flat_map(|(left, right, kind)| {
                    [
                        (left, kind, GroupingSide::Left),
                        (right, kind, GroupingSide::Right),
                    ]
                })
                .map(|(symbol, kind, side)| {
                    (
                        symbol,
                        Keyword::Symbol(Symbol::Grouping(Grouping { kind, side })),
                    )
                }),
        );

        // add function mappings to the keyword mappings
        keyword_mapping.extend(
            Self::FUNCTION_MAPPING
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
