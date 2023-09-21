#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Token<'a> {
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
    Equal(bool),
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
    Colon,
    In(bool),
    Sum,
    Integral,
    Function(Function),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Function {
    Sqrt,
    Sin,
    Cos,
    Tan,
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
