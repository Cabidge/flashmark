#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Token<'a> {
    Literal(Literal<'a>),
    Keyword(Keyword),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Literal<'a> {
    Identifier(&'a str),
    Text(&'a str),
    Number(&'a str),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Keyword {
    Symbol(Symbol),
    Function(Function),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Symbol {
    Plus,
    Minus,
    Slash,
    DotProduct,
    CrossProduct,
    Caret,
    Underscore,
    Grouping(Grouping),
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
    Colon,
    In,
    NotIn,
    Sum,
    Integral,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Grouping {
    pub kind: GroupingKind,
    pub side: GroupingSide,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum GroupingKind {
    Paren,
    Bracket,
    Brace,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum GroupingSide {
    Left,
    Right,
}

/// A token that is followed by a single argument.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Function {
    Sqrt,
    Sin,
    Cos,
    Tan,
}
