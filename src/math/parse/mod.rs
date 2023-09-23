pub mod expressions;
pub mod tokenize;

use std::iter::Peekable;

use expressions::{Expr, ExprVariant, GroupExpr, UnitExpr};
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
        let variant = self.parse_variant()?;

        let sub_script = self
            .token_stream
            .next_if_eq(&tokenize::token::Token::Keyword(
                tokenize::token::Keyword::Symbol(tokenize::token::Symbol::Special(
                    tokenize::token::SpecialSymbol::Underscore,
                )),
            ))
            .and_then(|_| self.parse_expr());

        let super_script = self
            .token_stream
            .next_if_eq(&tokenize::token::Token::Keyword(
                tokenize::token::Keyword::Symbol(tokenize::token::Symbol::Special(
                    tokenize::token::SpecialSymbol::Caret,
                )),
            ))
            .and_then(|_| self.parse_expr());

        Some(UnitExpr {
            variant,
            sub_script,
            super_script,
        })
    }

    fn parse_variant(&mut self) -> Option<ExprVariant> {
        use tokenize::token::{self, Token};

        let expr = match self.token_stream.next()? {
            Token::Literal(literal) => match literal {
                token::Literal::Variable(ch) => ExprVariant::Identifier(Box::from(ch.to_string())),
                token::Literal::Number(num) => ExprVariant::Num(num),
                token::Literal::Text(text) => ExprVariant::Text(text),
            },
            Token::Keyword(token::Keyword::Symbol(symbol)) => match symbol {
                token::Symbol::Simple(symbol) => todo!(),
                token::Symbol::Special(symbol) => match symbol {
                    token::SpecialSymbol::Grouping(grouping) => match grouping.side {
                        token::GroupingSide::Left => {
                            let body = self.parse_grouping(grouping.kind);
                            ExprVariant::Grouping(body)
                        }
                        token::GroupingSide::Right => todo!(),
                    },
                    _ => todo!(),
                },
            },
            Token::Keyword(token::Keyword::Function(function)) => todo!(),
        };

        Some(expr)
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
