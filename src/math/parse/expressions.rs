use super::tokenize::token::{Function, GroupingKind};

#[derive(Debug, Clone)]
pub enum Expr {
    Unit(Box<UnitExpr>),
    Fraction(Box<Fraction>),
}

#[derive(Debug, Clone)]
pub struct UnitExpr {
    pub variant: ExprVariant,
    pub super_script: Option<Expr>,
    pub sub_script: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct Fraction {
    pub numerator: Expr,
    pub denominator: Expr,
}

#[derive(Debug, Clone)]
pub enum ExprVariant {
    Identifier(Box<str>),
    Operator(Box<str>),
    Num(Box<str>),
    Text(Box<str>),
    Unary(Function, Box<Expr>),
    Grouping {
        left: GroupingKind,
        right: GroupingKind,
        body: Vec<Expr>,
    },
}
