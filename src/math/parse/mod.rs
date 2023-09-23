pub mod expressions;
pub mod tokenize;

use std::iter::Peekable;

use expressions::{Expr, GroupExpr, UnitExpr};
use tokenize::token::GroupingKind;

pub struct Parser<'a> {
    token_stream: Peekable<tokenize::Tokenizer<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            token_stream: tokenize::Tokenizer::new(input).peekable(),
        }
    }

    fn parse_expr(&mut self) -> Option<Expr> {
        let unit = self.parse_unit()?;

        let Some(_slash) = self
            .token_stream
            .next_if_eq(&tokenize::token::Token::Keyword(
                tokenize::token::Keyword::Symbol(tokenize::token::Symbol::Special(
                    tokenize::token::SpecialSymbol::Slash,
                )),
            ))
        else {
            return Some(Expr::Unit(Box::new(unit)));
        };

        // TODO: Handle incomplete fractions
        let denominator = self.parse_unit()?;

        Some(Expr::Fraction(Box::new(expressions::Fraction {
            numerator: Expr::Unit(Box::new(unit)),
            denominator: Expr::Unit(Box::new(denominator)),
        })))
    }

    fn parse_unit(&mut self) -> Option<UnitExpr> {
        todo!()
    }

    fn parse_grouping(&mut self, left: GroupingKind) -> GroupExpr {
        todo!()
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Expr;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_expr()
    }
}
