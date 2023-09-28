use super::{
    sanitize::LazySanitize,
    tokenize::token::{self, Function, GroupingKind},
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

impl From<token::Symbol> for ExprVariant {
    fn from(symbol: token::Symbol) -> Self {
        use token::Symbol::*;

        match symbol {
            Special(symbol) => symbol.into(),
            Simple(symbol) => symbol.into(),
        }
    }
}

impl From<token::SimpleSymbol> for ExprVariant {
    fn from(symbol: token::SimpleSymbol) -> Self {
        use token::SimpleSymbol::*;

        match symbol {
            Plus => ExprVariant::Operator("+".into()),
            Minus => ExprVariant::Operator("-".into()),
            DotProduct => ExprVariant::Operator("⋅".into()),
            CrossProduct => ExprVariant::Operator("×".into()),
            Equal => ExprVariant::Operator("=".into()),
            NotEqual => ExprVariant::Operator("≠".into()),
            LessThan => ExprVariant::Operator("<".into()),
            GreaterThan => ExprVariant::Operator(">".into()),
            LessThanOrEqual => ExprVariant::Operator("≤".into()),
            GreaterThanOrEqual => ExprVariant::Operator("≥".into()),
            Colon => ExprVariant::Operator(":".into()),
            In => ExprVariant::Operator("∈".into()),
            NotIn => ExprVariant::Operator("∉".into()),
            RightArrow => ExprVariant::Operator("→".into()),
            LeftArrow => ExprVariant::Operator("←".into()),
            Sum => ExprVariant::Operator("∑".into()),
            Integral => ExprVariant::Operator("∫".into()),
        }
    }
}

impl From<token::Grouping> for ExprVariant {
    fn from(token::Grouping { kind, side }: token::Grouping) -> Self {
        use token::{GroupingKind::*, GroupingSide::*};
        match (kind, side) {
            (Paren, Left) => ExprVariant::Operator("(".into()),
            (Paren, Right) => ExprVariant::Operator(")".into()),
            (Bracket, Left) => ExprVariant::Operator("[".into()),
            (Bracket, Right) => ExprVariant::Operator("]".into()),
            (Brace, Left) => ExprVariant::Operator("{".into()),
            (Brace, Right) => ExprVariant::Operator("}".into()),
        }
    }
}

impl From<token::SpecialSymbol> for ExprVariant {
    fn from(symbol: token::SpecialSymbol) -> Self {
        use token::SpecialSymbol::*;

        match symbol {
            Slash => ExprVariant::Operator("/".into()),
            Caret => ExprVariant::Operator("^".into()),
            Underscore => ExprVariant::Operator("_".into()),
            Grouping(grouping) => grouping.into(),
        }
    }
}
