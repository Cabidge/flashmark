use super::{
    sanitize::LazySanitize,
    tokenize::{Function, GroupingKind, Symbol},
};

#[derive(Debug, Clone)]
pub enum Expr {
    Unit(Box<UnitExpr>),
    Fraction(Box<Fraction>),
}

#[derive(Debug, Clone)]
pub struct UnitExpr {
    pub variant: ExprVariant,
    pub sub_script: Option<ExprVariant>,
    pub super_script: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct Fraction {
    pub numerator: Expr,
    pub denominator: Expr,
}

#[derive(Debug, Clone)]
pub enum ExprVariant {
    Identifier(LazySanitize),
    Operator(LazySanitize),
    Num(Box<str>),
    Text(LazySanitize),
    Unary(Function, Box<Expr>),
    Grouping(GroupExpr),
}

#[derive(Debug, Clone)]
pub struct GroupExpr {
    pub left: GroupingKind,
    pub right: GroupingKind,
    pub body: Vec<Expr>,
}

impl From<Symbol> for ExprVariant {
    fn from(symbol: Symbol) -> Self {
        let symbol_ch = char::from(symbol);
        ExprVariant::Operator(symbol_ch.into())
    }
}

impl GroupExpr {
    pub fn has_parens(&self) -> bool {
        self.left == GroupingKind::Paren && self.right == GroupingKind::Paren
    }
}
