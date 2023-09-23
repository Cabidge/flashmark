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
        todo!()
    }

    fn parse_unit(&mut self) -> Option<UnitExpr> {
        todo!()
    }

    fn parse_grouping(&mut self, left: GroupingKind) -> GroupExpr {
        todo!()
    }
}
