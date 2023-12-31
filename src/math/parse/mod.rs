pub mod expressions;
pub mod sanitize;
pub mod tokenize;

use std::iter::Peekable;

use expressions::{Expr, ExprVariant, GroupExpr, UnitExpr};
use tokenize::{GroupingKind, Keyword, Literal, SpecialSymbol, Token};

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
        const FRACTION_TOKEN: Token =
            Token::Keyword(Keyword::new_special_symbol(SpecialSymbol::Slash));

        let unit = self.parse_unit()?;

        let Some(_slash) = self.token_stream.next_if_eq(&FRACTION_TOKEN) else {
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
        const SUBSCRIPT_TOKEN: Token =
            Token::Keyword(Keyword::new_special_symbol(SpecialSymbol::Underscore));

        const SUPERSCRIPT_TOKEN: Token =
            Token::Keyword(Keyword::new_special_symbol(SpecialSymbol::Caret));

        let variant = self.parse_variant()?;

        let sub_script = self
            .token_stream
            .next_if_eq(&SUBSCRIPT_TOKEN)
            .and_then(|_| self.parse_variant());

        let super_script = self
            .token_stream
            .next_if_eq(&SUPERSCRIPT_TOKEN)
            .and_then(|_| self.parse_expr());

        Some(UnitExpr {
            variant,
            sub_script,
            super_script,
        })
    }

    fn parse_variant(&mut self) -> Option<ExprVariant> {
        let expr = match self.token_stream.next()? {
            Token::Literal(literal) => match literal {
                Literal::Variable(ch) => ExprVariant::Identifier(ch.into()),
                Literal::Number(num) => ExprVariant::Num(num),
                Literal::Text(text) => ExprVariant::Text(text.into()),
            },
            Token::Keyword(keyword) => {
                if let Some(grouping_kind) = keyword.left_grouping() {
                    let body = self.parse_grouping(grouping_kind);
                    ExprVariant::Grouping(body)
                } else {
                    match keyword {
                        Keyword::Symbol(symbol) => ExprVariant::from(symbol),
                        Keyword::Function(function) => {
                            let expr = self.parse_expr()?;
                            ExprVariant::Unary(function, Box::new(expr))
                        }
                    }
                }
            }
        };

        Some(expr)
    }

    fn parse_grouping(&mut self, left: GroupingKind) -> GroupExpr {
        let mut body = vec![];

        let right = loop {
            let Some(next_token) = self.token_stream.peek() else {
                break GroupingKind::Paren;
            };

            if let Token::Keyword(keyword) = next_token {
                if let Some(kind) = keyword.right_grouping() {
                    self.token_stream.next(); // Consume the token
                    break kind;
                }
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
