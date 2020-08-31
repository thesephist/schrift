#[derive(Debug)]
pub enum Tok {
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

pub fn tokenize(prog: &str) -> Vec<Tok> {
    return Vec::<Tok>::new();
}
