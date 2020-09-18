use std::fmt;

use crate::err::InkErr;

#[derive(Debug, PartialEq, Clone)]
pub enum TokKind {
    Separator,

    Comment(String),

    Ident(String),
    EmptyIdent,

    NumberLiteral(f64),
    StringLiteral(String),

    TrueLiteral,
    FalseLiteral,

    AccessorOp,

    EqOp,
    FunctionArrow,

    KeyValueSeparator,
    DefineOp,
    MatchColon,

    CaseArrow,
    SubOp,

    NegOp,
    AddOp,
    MulOp,
    DivOp,
    ModOp,
    GtOp,
    LtOp,

    AndOp,
    OrOp,
    XorOp,

    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
}

#[derive(Debug, Clone)]
pub struct Span(usize, usize);

#[derive(Debug, Clone)]
pub struct Tok<'s> {
    pub kind: TokKind,
    pub span: Span,
    source: &'s str,
}

impl fmt::Display for Tok<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pos = self.position();
        write!(f, "{:?} [{}:{}]", self.kind, pos.line, pos.col,)
    }
}

#[derive(Debug)]
pub struct Position {
    line: usize,
    col: usize,
}

impl<'s> Tok<'s> {
    fn position(&self) -> Position {
        // first get to right line
        let mut line: usize = 1;
        let mut col: usize = 1;
        for c in self.source[0..self.span.0].chars() {
            if c == '\n' {
                line += 1;
                col = 0;
            }
            col += 1;
        }
        // then count columns
        return Position { line, col };
    }
}

#[derive(Debug)]
pub struct Reader<'s> {
    source: &'s str,
    start: usize,
    index: usize,
}

// impl should be more efficient. In particular, peek()
// should support seeking thru Unicode source text in constant time.
impl<'s> Reader<'s> {
    fn new(source: &str) -> Reader {
        return Reader {
            source,
            start: 0,
            index: 0,
        };
    }

    fn peek(&self) -> char {
        return self
            .source
            .chars()
            .nth(self.index)
            .expect("Reader index out of bounds in peek");
    }

    fn lookback(&self) -> char {
        return self
            .source
            .chars()
            .nth(self.index - 1)
            .expect("Reader index out of bounds in lookback");
    }

    fn next(&mut self) {
        if self.has_next() {
            self.index += 1;
        }
    }

    fn has_next(&self) -> bool {
        return self.source.len() > self.index;
    }

    fn pop_span(&mut self) -> Span {
        let span = Span(self.start, self.index);
        self.start = self.index;
        return span;
    }

    fn pop_token(&mut self, kind: TokKind) -> Tok<'s> {
        return Tok {
            kind: kind,
            span: self.pop_span(),
            source: self.source,
        };
    }

    fn pop_token_and_next(&mut self, kind: TokKind) -> Tok<'s> {
        self.next();
        return Tok {
            kind: kind,
            span: self.pop_span(),
            source: self.source,
        };
    }

    fn take_while<F>(&mut self, cond: F) -> &str
    where
        F: Fn(char) -> bool,
    {
        while self.has_next() && (cond(self.peek()) || self.lookback() == '\\') {
            self.next();
        }
        return self.take();
    }

    fn take_until<F>(&mut self, cond: F) -> &str
    where
        F: Fn(char) -> bool,
    {
        while self.has_next() && (cond(self.peek()) || self.lookback() == '\\') {
            self.next();
        }
        self.next();
        return self.take();
    }

    fn take(&self) -> &str {
        return &self.source[self.start..self.index];
    }
}

pub fn tokenize(prog: &str) -> Result<Vec<Tok>, InkErr> {
    let mut tokens = Vec::<Tok>::new();
    let mut reader = Reader::new(prog);

    fn ensure_separator<'s>(tokens: &mut Vec<Tok<'s>>, reader: &mut Reader<'s>) {
        match tokens.last() {
            Some(tok) => match tok.kind {
                TokKind::Separator
                | TokKind::Comment(_)
                | TokKind::LParen
                | TokKind::LBracket
                | TokKind::LBrace
                | TokKind::AddOp
                | TokKind::SubOp
                | TokKind::MulOp
                | TokKind::DivOp
                | TokKind::ModOp
                | TokKind::NegOp
                | TokKind::GtOp
                | TokKind::LtOp
                | TokKind::EqOp
                | TokKind::DefineOp
                | TokKind::AccessorOp
                | TokKind::KeyValueSeparator
                | TokKind::FunctionArrow
                | TokKind::MatchColon
                | TokKind::CaseArrow => (),
                _ => tokens.push(reader.pop_token(TokKind::Separator)),
            },
            None => return,
        };
    }

    while reader.has_next() {
        let c = reader.peek();

        match c {
            '\'' => {
                reader.next(); // opening quote
                reader.pop_span();

                let str_content = reader.take_while(|c| c != '\'');
                let str_value = String::from(str_content);
                tokens.push(reader.pop_token(TokKind::StringLiteral(str_value)));

                reader.next(); // closing quote
            }
            '`' => {
                reader.next(); // opening backtick

                if reader.peek() == '`' {
                    ensure_separator(&mut tokens, &mut reader);

                    // line comment
                    reader.next(); // second backtick
                    reader.pop_span();

                    let str_content = reader.take_until(|c| c != '\n');
                    let str_value = String::from(str_content);
                    tokens.push(reader.pop_token(TokKind::Comment(str_value)));

                    reader.next(); // newline
                } else {
                    // block comment
                    reader.pop_span();

                    let str_content = reader.take_while(|c| c != '`');
                    let str_value = String::from(str_content);
                    tokens.push(reader.pop_token(TokKind::Comment(str_value)));

                    reader.next(); // closing backtick
                }
            }
            '\n' => {
                ensure_separator(&mut tokens, &mut reader);
                reader.next();
                reader.pop_span();
            }
            '\t' => {
                reader.next();
                reader.pop_span();
            }
            ' ' => {
                reader.next();
                reader.pop_span();
            }
            '_' => tokens.push(reader.pop_token_and_next(TokKind::EmptyIdent)),
            '~' => tokens.push(reader.pop_token_and_next(TokKind::NegOp)),
            '+' => tokens.push(reader.pop_token_and_next(TokKind::AddOp)),
            '*' => tokens.push(reader.pop_token_and_next(TokKind::MulOp)),
            '/' => tokens.push(reader.pop_token_and_next(TokKind::DivOp)),
            '%' => tokens.push(reader.pop_token_and_next(TokKind::ModOp)),
            '&' => tokens.push(reader.pop_token_and_next(TokKind::AndOp)),
            '|' => tokens.push(reader.pop_token_and_next(TokKind::OrOp)),
            '^' => tokens.push(reader.pop_token_and_next(TokKind::XorOp)),
            '<' => tokens.push(reader.pop_token_and_next(TokKind::LtOp)),
            '>' => tokens.push(reader.pop_token_and_next(TokKind::GtOp)),
            ',' => tokens.push(reader.pop_token_and_next(TokKind::Separator)),
            '.' => tokens.push(reader.pop_token_and_next(TokKind::AccessorOp)),
            '(' => tokens.push(reader.pop_token_and_next(TokKind::LParen)),
            ')' => {
                ensure_separator(&mut tokens, &mut reader);
                tokens.push(reader.pop_token_and_next(TokKind::RParen));
            }
            '[' => tokens.push(reader.pop_token_and_next(TokKind::LBracket)),
            ']' => {
                ensure_separator(&mut tokens, &mut reader);
                tokens.push(reader.pop_token_and_next(TokKind::RBracket));
            }
            '{' => tokens.push(reader.pop_token_and_next(TokKind::LBrace)),
            '}' => {
                ensure_separator(&mut tokens, &mut reader);
                tokens.push(reader.pop_token_and_next(TokKind::RBrace));
            }
            ':' => {
                reader.next();
                match reader.peek() {
                    ':' => {
                        tokens.push(reader.pop_token_and_next(TokKind::MatchColon));
                    }
                    '=' => {
                        tokens.push(reader.pop_token_and_next(TokKind::DefineOp));
                    }
                    _ => tokens.push(reader.pop_token_and_next(TokKind::KeyValueSeparator)),
                }
            }
            '=' => {
                reader.next();
                match reader.peek() {
                    '>' => {
                        tokens.push(reader.pop_token_and_next(TokKind::FunctionArrow));
                    }
                    _ => tokens.push(reader.pop_token_and_next(TokKind::EqOp)),
                }
            }
            '-' => {
                reader.next();
                match reader.peek() {
                    '>' => {
                        tokens.push(reader.pop_token_and_next(TokKind::CaseArrow));
                    }
                    _ => tokens.push(reader.pop_token_and_next(TokKind::SubOp)),
                }
            }
            '0'..='9' => {
                let numeral = reader.take_while(|c| c >= '0' && c <= '9' || c == '.');
                let r = numeral.parse::<f64>();
                match r {
                    Ok(num) => tokens.push(reader.pop_token(TokKind::NumberLiteral(num))),
                    Err(_) => return Err(InkErr::InvalidNumber(String::from(numeral))),
                }
            }
            _ => {
                // TODO support full unicode
                let ident = reader
                    .take_while(|c| c.is_ascii_alphanumeric() || c == '?' || c == '!' || c == '@');

                let ident_bit = String::from(ident);
                match ident {
                    "true" => tokens.push(reader.pop_token(TokKind::TrueLiteral)),
                    "false" => tokens.push(reader.pop_token(TokKind::FalseLiteral)),
                    _ => tokens.push(reader.pop_token(TokKind::Ident(ident_bit))),
                }
            }
        }
    }

    return Ok(tokens);
}
