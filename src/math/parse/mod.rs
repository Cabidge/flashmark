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
        use tokenize::token::{GroupingSide, Keyword, Literal, SpecialSymbol, Symbol, Token};

        let expr = match self.token_stream.next()? {
            Token::Literal(literal) => match literal {
                Literal::Variable(ch) => ExprVariant::Identifier(Box::from(ch.to_string())),
                Literal::Number(num) => ExprVariant::Num(num),
                Literal::Text(text) => ExprVariant::Text(text),
            },
            Token::Keyword(keyword) => match keyword {
                Keyword::Symbol(symbol) => match symbol {
                    Symbol::Special(SpecialSymbol::Grouping(grouping))
                        if grouping.side == GroupingSide::Left =>
                    {
                        let body = self.parse_grouping(grouping.kind);
                        ExprVariant::Grouping(body)
                    }
                    symbol => ExprVariant::from(symbol),
                },
                Keyword::Function(function) => {
                    let expr = self.parse_expr()?;
                    ExprVariant::Unary(function, Box::new(expr))
                }
            },
        };

        Some(expr)
    }

    fn parse_grouping(&mut self, left: GroupingKind) -> GroupExpr {
        use tokenize::token::{GroupingSide, Keyword::*, SpecialSymbol::*, Symbol::*, Token};

        let mut body = vec![];

        let right = loop {
            let Some(next_token) = self.token_stream.peek() else {
                break GroupingKind::Paren;
            };

            match next_token {
                Token::Keyword(Symbol(Special(Grouping(grouping))))
                    if grouping.side == GroupingSide::Right =>
                {
                    break grouping.kind
                }
                _ => (),
            }

            let Some(expr) = self.parse_expr() else {
                break GroupingKind::Paren;
            };

            body.push(expr);
        };

        GroupExpr { left, right, body }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Expr;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_expr()
    }
}
