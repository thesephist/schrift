use std::fmt;

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
    kind: TokKind,
    span: Span,
    source: &'s str,
}

impl fmt::Display for Tok<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?} [{}]",
            self.kind,
            &self.source[self.span.0..self.span.1]
        )
    }
}

#[derive(Debug)]
pub enum LexError {
    InvalidNumber(String),
}

// TODO: support producing Spans with line/column info
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
            .expect("Reader index out of bounds");
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
        while cond(self.peek()) {
            self.next();
        }
        return self.take();
    }

    fn take(&self) -> &str {
        return &self.source[self.start..self.index];
    }
}

pub fn tokenize(prog: &str) -> Result<Vec<Tok>, LexError> {
    let mut tokens = Vec::<Tok>::new();
    let mut reader = Reader::new(prog);

    fn ensure_separator<'s>(tokens: &mut Vec<Tok<'s>>, reader: &mut Reader<'s>) {
        match tokens.last() {
            Some(tok) => {
                if tok.kind != TokKind::Separator {
                    tokens.push(reader.pop_token(TokKind::Separator));
                }
            }
            None => return,
        };
    }

    while reader.has_next() {
        let c = reader.peek();

        match c {
            '\'' => {
                println!("start of string");
                reader.next();
            }
            '`' => {
                println!("start of comment");
                reader.next();
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
            ')' => tokens.push(reader.pop_token_and_next(TokKind::RParen)),
            '[' => tokens.push(reader.pop_token_and_next(TokKind::LBracket)),
            ']' => tokens.push(reader.pop_token_and_next(TokKind::RBracket)),
            '{' => tokens.push(reader.pop_token_and_next(TokKind::LBrace)),
            '}' => tokens.push(reader.pop_token_and_next(TokKind::RBrace)),
            ':' => {
                reader.next();
                match reader.peek() {
                    ':' => {
                        tokens.push(reader.pop_token_and_next(TokKind::MatchColon));
                    }
                    '=' => {
                        tokens.push(reader.pop_token_and_next(TokKind::MatchColon));
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
                    Err(_) => return Err(LexError::InvalidNumber(String::from(numeral))),
                }
            }
            _ => {
                // TODO support full unicode
                let ident = reader
                    .take_while(|c| c.is_ascii_alphanumeric() || c == '?' || c == '!' || c == '@');
                let ident_bit = String::from(ident);
                tokens.push(reader.pop_token(TokKind::Ident(ident_bit)));
            }
        }
    }

    return Ok(tokens);
}
