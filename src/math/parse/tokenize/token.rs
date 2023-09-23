#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    Literal(Literal),
    Keyword(Keyword),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Literal {
    Variable(char),
    Text(Box<str>),
    Number(Box<str>),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Keyword {
    Symbol(Symbol),
    Function(Function),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Symbol {
    Special(SpecialSymbol),
    Simple(SimpleSymbol),
}

/// A symbol with extra meaning that doesn't translate directly to a single expression.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SpecialSymbol {
    Slash,
    Caret,
    Underscore,
    Grouping(Grouping),
}

/// A symbol that translates directly to a single expression.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SimpleSymbol {
    Plus,
    Minus,
    DotProduct,
    CrossProduct,
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

impl Keyword {
    pub fn new_simple_symbol(symbol: SimpleSymbol) -> Self {
        Self::Symbol(Symbol::Simple(symbol))
    }

    pub fn new_special_symbol(symbol: SpecialSymbol) -> Self {
        Self::Symbol(Symbol::Special(symbol))
    }

    pub fn new_grouping(kind: GroupingKind, side: GroupingSide) -> Self {
        Self::new_special_symbol(SpecialSymbol::Grouping(Grouping { kind, side }))
    }
}
