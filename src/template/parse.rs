use std::iter::Peekable;

pub type Block = Vec<Result<Stmt, rhai::ParseError>>;

#[derive(Debug, Clone)]
pub struct Parser<'a> {
    pub state: ParserState<'a>,
    current_step: Option<ParserStep>,
}

#[derive(Debug, Clone)]
pub struct ParserState<'a> {
    pub engine: &'a rhai::Engine,
    pub scope: &'a rhai::Scope<'static>,
    pub chars: CharStream<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParserStep {
    Literal(String),
    Expr,
    If,
    For,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(rhai::AST),
    If(IfChainStmt),
    For(ForStmt),
    Literal(String),
}

#[derive(Debug, Clone)]
pub struct IfChainStmt {
    pub ifs: Vec<IfStmt>,
    pub tail: Option<Block>,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub expr: rhai::AST,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub name: String,
    pub expr: rhai::AST,
    pub body: Block,
}

type CharStream<'a> = Peekable<std::str::Chars<'a>>;

type StepResult = (Option<Result<Stmt, rhai::ParseError>>, Option<ParserStep>);

pub fn parse_front_matter(input: &str) -> (Option<&str>, &str) {
    input
        .strip_prefix("---\n")
        .and_then(|stripped| stripped.split_once("---\n"))
        .map(|(front, input)| (Some(front), input))
        .unwrap_or((None, input))
}

impl<'a> Parser<'a> {
    pub fn new(engine: &'a rhai::Engine, scope: &'a rhai::Scope<'static>, input: &'a str) -> Self {
        Self::with_state(ParserState {
            engine,
            scope,
            chars: input.chars().peekable(),
        })
    }

    fn with_state(state: ParserState<'a>) -> Self {
        Self {
            state,
            current_step: Some(ParserStep::Literal(String::new())),
        }
    }

    pub fn parse_step(&mut self) -> Option<Result<Stmt, rhai::ParseError>> {
        let current_state = self.current_step.take();
        let (res, next_state) = Self::step(current_state, &mut self.state);

        self.current_step = next_state;

        // don't emit empty literals
        if matches!(&res, Some(Ok(Stmt::Literal(lit))) if lit.is_empty()) {
            None
        } else {
            res
        }
    }

    fn step(current_step: Option<ParserStep>, state: &mut ParserState) -> StepResult {
        let Some(current_step) = current_step else {
            return (None, None);
        };

        match current_step {
            ParserStep::Literal(literal) => Self::step_literal(literal, state),
            ParserStep::Expr => {
                state.chars.next(); // consume the '('

                let expr = state
                    .chars
                    .by_ref()
                    .take_while(|&c| c != ')')
                    .collect::<String>();

                let stmt = state.compile_expression(expr).map(Stmt::Expr);

                (Some(stmt), Some(ParserStep::Literal(String::new())))
            }
            ParserStep::If => {
                let res = capture_if_chain_stmt(state).map(Stmt::If);
                (Some(res), Some(ParserStep::Literal(String::new())))
            }
            ParserStep::For => {
                let res = capture_for_stmt(state).map(Stmt::For);
                (Some(res), Some(ParserStep::Literal(String::new())))
            }
        }
    }

    fn step_literal(mut literal: String, state: &mut ParserState) -> StepResult {
        let Some(c) = state.chars.next() else {
            return (Some(Ok(Stmt::Literal(literal))), None);
        };

        match (c, state.chars.peek().copied()) {
            // capture expression
            ('@', Some('(')) => (Some(Ok(Stmt::Literal(literal))), Some(ParserStep::Expr)),
            // capture keyword
            ('@', Some(c_next)) if c_next.is_alphabetic() => {
                let keyword = capture_keyword(&mut state.chars);

                let next_state = match keyword.as_str() {
                    "if" if state.engine.allow_if_expression() => Some(ParserStep::If),
                    "for" if state.engine.allow_looping() => Some(ParserStep::For),
                    _ => None,
                };

                if let Some(state) = next_state {
                    (Some(Ok(Stmt::Literal(literal))), Some(state))
                } else {
                    use std::fmt::Write;
                    write!(literal, "{c}{keyword} ").expect("String should not fail to write");
                    (None, Some(ParserStep::Literal(literal)))
                }
            }
            _ => {
                literal.push(c);
                (None, Some(ParserStep::Literal(literal)))
            }
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<Stmt, rhai::ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        // keep parsing until we get a statement or run out of characters
        loop {
            // if state is None, we've run out of characters
            self.current_step.as_ref()?;

            if let Some(stmt) = self.parse_step() {
                return Some(stmt);
            }
        }
    }
}

impl<'a> ParserState<'a> {
    pub fn compile_expression(&self, expr: impl AsRef<str>) -> Result<rhai::AST, rhai::ParseError> {
        self.engine.compile_expression_with_scope(self.scope, expr)
    }
}

impl ParserStep {
    pub fn literal(self) -> Option<String> {
        match self {
            Self::Literal(lit) => Some(lit),
            _ => None,
        }
    }
}

/// Assumes the opening '@' has already been consumed.
/// Captures @<keyword>.
fn capture_keyword(chars: &mut CharStream) -> String {
    let mut keyword = String::new();
    while let Some(c) = chars.next_if(|c| c.is_alphabetic()) {
        keyword.push(c);
    }
    keyword
}

/// Assumes the opening '{' has already been consumed.
/// Captures { <body> }.
///
/// Returns the parsed body.
fn capture_body(state: &mut ParserState) -> Block {
    // look ahead for a newline
    'look_ahead: {
        let mut look_ahead = state.chars.clone();

        if look_ahead.find(|&ch| !ch.is_whitespace() || ch == '\n') != Some('\n') {
            break 'look_ahead;
        }

        look_ahead.next_if_eq(&'\r');

        state.chars = look_ahead;
    }

    let mut nested_parser = Parser::with_state((*state).clone());

    let mut block = vec![];
    // parse until we consume the closing '}'
    while nested_parser.state.chars.next_if(|&c| c == '}').is_none() {
        if nested_parser.current_step.is_none() {
            break;
        }

        if let Some(stmt) = nested_parser.parse_step() {
            block.push(stmt);
        }
    }

    // remove trailing newline
    if let Some(mut literal) = nested_parser
        .current_step
        .and_then(ParserStep::literal)
        .filter(|s| !s.is_empty())
    {
        if let Some((before, _)) = literal
            .rsplit_once('\n')
            .filter(|(_, after)| after.trim().is_empty())
        {
            literal.truncate(before.len());
        }

        block.push(Ok(Stmt::Literal(literal)));
    }

    // outer parser continues from where the nested parser left off
    *state = nested_parser.state;

    block
}

/// Assumes a preceeding '@if' or '@elif' has already been consumed.
/// Captures @if/@elif <expr> { <body> }.
fn capture_if_stmt(state: &mut ParserState) -> Result<IfStmt, rhai::ParseError> {
    let expr = state
        .chars
        .by_ref()
        .take_while(|&c| c != '{')
        .collect::<String>();

    let expr = state.compile_expression(expr)?;

    let body = capture_body(state);

    Ok(IfStmt { expr, body })
}

/// Assumes a preceeding '@if' has already been consumed.
/// Captures @if <expr> { <body> } [@elif  <expr> { <body> }]* [@else { <body> }]
fn capture_if_chain_stmt(state: &mut ParserState) -> Result<IfChainStmt, rhai::ParseError> {
    let head = capture_if_stmt(state)?;

    let mut stmt = IfChainStmt {
        ifs: vec![head],
        tail: None,
    };

    // look ahead to find elif or else
    loop {
        let mut look_ahead = state.chars.clone();

        if look_ahead.find(|c| !c.is_whitespace()) != Some('@') {
            break;
        }

        let keyword = capture_keyword(&mut look_ahead);

        match keyword.as_str() {
            "elif" => {
                // no longer need to look ahead
                state.chars = look_ahead;

                let elif = capture_if_stmt(state)?;
                stmt.ifs.push(elif);
            }
            "else" => {
                // no longer need to look ahead
                state.chars = look_ahead;

                // consume until '{'
                state.chars.find(|&c| c == '{');

                let body = capture_body(state);
                stmt.tail = Some(body);

                // no more elif or else
                break;
            }
            _ => break,
        }
    }

    Ok(stmt)
}

/// Assumes a preceeding '@for' has already been consumed.
/// Captures @for <name> in <expr> { <body> }.
fn capture_for_stmt(state: &mut ParserState) -> Result<ForStmt, rhai::ParseError> {
    let header = state
        .chars
        .by_ref()
        .take_while(|&c| c != '{')
        .collect::<String>();

    let Some((name, expr)) = header.trim().split_once(" in ") else {
        let err_type = rhai::ParseErrorType::MissingToken(String::from("in"), header);
        return Err(rhai::ParseError(err_type.into(), rhai::Position::NONE));
    };

    let expr = state.compile_expression(expr)?;

    let body = capture_body(state);

    Ok(ForStmt {
        name: name.to_string(),
        expr,
        body,
    })
}

#[cfg(test)]
mod tests {}
