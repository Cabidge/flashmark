#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Token<'a> {
    Unit(Unit<'a>),
    Function(Function),
}

/// A single token that takes no arguments.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Unit<'a> {
    Identifier(&'a str),
    Text(&'a str),
    Number(&'a str),
    Symbol(Symbol),
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
    Grouping(Grouping, GroupingKind),
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
pub enum Grouping {
    Paren,
    Bracket,
    Brace,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum GroupingKind {
    Open,
    Close,
}

/// A token that is followed by a single argument.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Function {
    Sqrt,
    Sin,
    Cos,
    Tan,
}
