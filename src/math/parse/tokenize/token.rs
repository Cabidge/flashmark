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
    RightArrow,
    LeftArrow,
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
    pub const fn new_simple_symbol(symbol: SimpleSymbol) -> Self {
        Self::Symbol(Symbol::Simple(symbol))
    }

    pub const fn new_special_symbol(symbol: SpecialSymbol) -> Self {
        Self::Symbol(Symbol::Special(symbol))
    }

    pub const fn new_grouping(kind: GroupingKind, side: GroupingSide) -> Self {
        Self::new_special_symbol(SpecialSymbol::Grouping(Grouping { kind, side }))
    }

    pub const fn grouping(self) -> Option<Grouping> {
        match self {
            Self::Symbol(Symbol::Special(SpecialSymbol::Grouping(grouping))) => Some(grouping),
            _ => None,
        }
    }

    pub fn left_grouping(self) -> Option<GroupingKind> {
        self.grouping().and_then(|grouping| match grouping.side {
            GroupingSide::Left => Some(grouping.kind),
            GroupingSide::Right => None,
        })
    }

    pub fn right_grouping(self) -> Option<GroupingKind> {
        self.grouping().and_then(|grouping| match grouping.side {
            GroupingSide::Right => Some(grouping.kind),
            GroupingSide::Left => None,
        })
    }
}

impl From<Symbol> for char {
    fn from(symbol: Symbol) -> Self {
        use Symbol::*;
        match symbol {
            Special(symbol) => symbol.into(),
            Simple(symbol) => symbol.into(),
        }
    }
}

impl From<SimpleSymbol> for char {
    fn from(symbol: SimpleSymbol) -> Self {
        use SimpleSymbol::*;
        match symbol {
            Plus => '+',
            Minus => '-',
            DotProduct => '⋅',
            CrossProduct => '×',
            Equal => '=',
            NotEqual => '≠',
            LessThan => '<',
            GreaterThan => '>',
            LessThanOrEqual => '≤',
            GreaterThanOrEqual => '≥',
            Colon => ':',
            In => '∈',
            NotIn => '∉',
            RightArrow => '→',
            LeftArrow => '←',
            Sum => '∑',
            Integral => '∫',
        }
    }
}

impl From<SpecialSymbol> for char {
    fn from(symbol: SpecialSymbol) -> Self {
        use SpecialSymbol::*;
        match symbol {
            Slash => '/',
            Caret => '^',
            Underscore => '_',
            Grouping(grouping) => char::from(grouping),
        }
    }
}

impl From<Grouping> for char {
    fn from(grouping: Grouping) -> Self {
        grouping.kind.into_char(grouping.side)
    }
}

impl GroupingKind {
    pub fn into_char(self, side: GroupingSide) -> char {
        use GroupingSide::*;
        match side {
            Left => self.into_left_char(),
            Right => self.into_right_char(),
        }
    }

    pub fn into_left_char(self) -> char {
        use GroupingKind::*;
        match self {
            Paren => '(',
            Bracket => '[',
            Brace => '{',
        }
    }

    pub fn into_right_char(self) -> char {
        use GroupingKind::*;
        match self {
            Paren => ')',
            Bracket => ']',
            Brace => '}',
        }
    }
}
