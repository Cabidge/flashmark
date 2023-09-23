use super::tokenize::token::{Function, GroupingKind};

pub enum Expr {
    Unit {
        variant: ExprVariant,
        super_script: Option<Box<Expr>>,
        sub_script: Option<Box<Expr>>,
    },
    Fraction {
        numerator: Box<Expr>,
        denominator: Box<Expr>,
    },
}

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
