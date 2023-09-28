use crate::parsing::StrParser;

use self::token::{
    Function, GroupingKind, GroupingSide, Keyword, Literal, SimpleSymbol, SpecialSymbol, Token,
};

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

    const SIMPLE_SYMBOL_MAPPING: &[(&'static str, token::SimpleSymbol)] = &[
        ("+", SimpleSymbol::Plus),
        ("-", SimpleSymbol::Minus),
        ("*", SimpleSymbol::DotProduct),
        ("xx", SimpleSymbol::CrossProduct),
        ("=", SimpleSymbol::Equal),
        ("!=", SimpleSymbol::NotEqual),
        ("<", SimpleSymbol::LessThan),
        (">", SimpleSymbol::GreaterThan),
        ("<=", SimpleSymbol::LessThanOrEqual),
        (">=", SimpleSymbol::GreaterThanOrEqual),
        (":", SimpleSymbol::Colon),
        ("in", SimpleSymbol::In),
        ("notin", SimpleSymbol::NotIn),
        ("->", SimpleSymbol::RightArrow),
        ("<-", SimpleSymbol::LeftArrow),
        ("sum", SimpleSymbol::Sum),
        ("int", SimpleSymbol::Integral),
    ];

    const SPECIAL_SYMBOL_MAPPING: &[(&'static str, SpecialSymbol)] = &[
        ("/", SpecialSymbol::Slash),
        ("^", SpecialSymbol::Caret),
        ("_", SpecialSymbol::Underscore),
    ];

    const GROUPING_MAPPING: &[(&'static str, &'static str, GroupingKind)] = &[
        ("(", ")", GroupingKind::Paren),
        ("[", "]", GroupingKind::Bracket),
        ("{", "}", GroupingKind::Brace),
    ];

    const FUNCTION_MAPPING: &[(&'static str, Function)] = &[
        ("sqrt", Function::Sqrt),
        ("sin", Function::Sin),
        ("cos", Function::Cos),
        ("tan", Function::Tan),
    ];

    fn keyword_mapping() -> Vec<(&'static str, Keyword)> {
        let mut keyword_mapping = vec![];

        keyword_mapping.extend(
            Self::SIMPLE_SYMBOL_MAPPING
                .iter()
                .map(|&(keyword, symbol)| (keyword, Keyword::new_simple_symbol(symbol))),
        );

        keyword_mapping.extend(
            Self::SPECIAL_SYMBOL_MAPPING
                .iter()
                .map(|&(keyword, symbol)| (keyword, Keyword::new_special_symbol(symbol))),
        );

        keyword_mapping.extend(
            Self::GROUPING_MAPPING
                .iter()
                .flat_map(|&(left, right, kind)| {
                    [
                        (left, kind, GroupingSide::Left),
                        (right, kind, GroupingSide::Right),
                    ]
                })
                .map(|(keyword, kind, side)| (keyword, Keyword::new_grouping(kind, side))),
        );

        keyword_mapping.extend(
            Self::FUNCTION_MAPPING
                .iter()
                .map(|&(keyword, function)| (keyword, Keyword::Function(function))),
        );

        keyword_mapping
    }

    fn try_tokenize_keyword(&mut self) -> Option<Keyword> {
        let mut keyword_mapping = Self::keyword_mapping();

        let mut parser = self.parser.clone();

        // remember the latest keyword match and the parser state
        let mut last_match: Option<(Keyword, StrParser<'a>)> = None;

        // loop until we have no more keywords to match
        while !keyword_mapping.is_empty() {
            let Some(ch) = parser.advance() else {
                break;
            };

            // remove all keywords that don't match the current character
            keyword_mapping.retain_mut(|(symbol, _)| {
                let Some(rest) = symbol.strip_prefix(ch) else {
                    return false;
                };
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

    fn try_tokenize_number(&mut self) -> Option<Box<str>> {
        // infinity
        if self.parser.consume_str("oo") {
            return Some(Box::from("∞"));
        }

        let num = self.parser.consume_while(|ch| ch.is_ascii_digit());
        (!num.is_empty()).then(|| Box::from(num))
    }

    fn try_tokenize_text(&mut self) -> Option<Box<str>> {
        if !self.parser.consume('"') {
            return None;
        }

        let text = Box::from(self.parser.consume_while(|ch| ch != '"'));

        self.parser.advance(); // consume the closing quote

        Some(text)
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.parser.skip_whitespace();

        if self.parser.is_exhausted() {
            return None;
        }

        if let Some(num) = self.try_tokenize_number() {
            return Some(Token::Literal(Literal::Number(num)));
        }

        if let Some(keyword) = self.try_tokenize_keyword() {
            return Some(Token::Keyword(keyword));
        }

        if let Some(text) = self.try_tokenize_text() {
            return Some(Token::Literal(Literal::Text(text)));
        }

        Some(Token::Literal(Literal::Variable(self.parser.advance()?)))
    }
}
