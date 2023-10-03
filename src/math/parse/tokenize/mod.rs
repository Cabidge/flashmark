pub mod token;
use std::collections::BTreeMap;

pub use token::*;

use crate::parsing::StrParser;

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

    fn keyboard_mapping() -> &'static BTreeMap<&'static str, Keyword> {
        use std::sync::OnceLock;

        static KEYWORD_MAPPING: OnceLock<BTreeMap<&'static str, Keyword>> = OnceLock::new();

        KEYWORD_MAPPING.get_or_init(Self::init_keyword_mapping)
    }

    fn init_keyword_mapping() -> BTreeMap<&'static str, Keyword> {
        let mut keyword_mapping = BTreeMap::new();

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
        let input = self.parser.rest();

        let min_slice = {
            let ch = input.chars().next()?;
            &input[..ch.len_utf8()]
        };

        let (skip_amount, keyword) = Self::keyboard_mapping()
            .range(min_slice..=input)
            .rev()
            // This stops the search early if it finds a keyword that was longer
            // than the previously checked keyword.
            //
            // The reason this is fine is because of how strings are sorted.
            .scan(usize::MAX, |prev_len, (&word, &keyword)| {
                if *prev_len >= word.len() {
                    *prev_len = word.len();
                    Some((word, keyword))
                } else {
                    None
                }
            })
            .find_map(|(word, keyword)| input.starts_with(word).then_some((word.len(), keyword)))?;

        self.parser.advance_by(skip_amount);

        Some(keyword)
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
