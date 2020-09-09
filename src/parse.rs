use crate::err::InkErr;
use crate::lex::{Tok, TokKind};

#[derive(Debug, Clone)]
pub enum Node {
    UnaryExpr {
        op: TokKind,
        arg: Box<Node>,
    },
    BinaryExpr {
        op: TokKind,
        left: Box<Node>,
        right: Box<Node>,
    },

    FnCall {
        func: Box<Node>,
        args: Vec<Node>,
    },

    MatchClause {
        target: Box<Node>,
        expr: Box<Node>,
    },
    MatchExpr {
        cond: Box<Node>,
        clauses: Vec<Node>,
    },
    ExprList(Vec<Node>),

    EmptyIdent,
    Ident(String),

    NumberLiteral(f64),
    StringLiteral(String),
    BooleanLiteral(bool),

    ObjectLiteral(Vec<Node>),
    ObjectEntry {
        key: Box<Node>,
        val: Box<Node>,
    },
    ListLiteral(Vec<Node>),
    FnLiteral {
        args: Vec<Node>,
        body: Box<Node>,
    },
}

type ParseResult = Result<Vec<Node>, InkErr>;

pub fn parse(tokens: Vec<Tok>) -> ParseResult {
    let tokens_without_comments: Vec<Tok> = tokens
        .into_iter()
        .filter(|tok| match tok.kind {
            TokKind::Comment(_) => true,
            _ => false,
        })
        .collect();

    let mut parser = Parser::new(tokens_without_comments);
    return parser.parse();
}

struct Parser<'s> {
    tokens: Vec<Tok<'s>>,
    nodes: Vec<Node>,
    idx: usize,
}

impl<'s> Parser<'s> {
    fn new(tokens: Vec<Tok>) -> Parser {
        return Parser {
            tokens,
            nodes: Vec::<Node>::new(),
            idx: 0,
        };
    }

    fn parse(&mut self) -> ParseResult {
        while self.idx < self.tokens.len() {
            self.parse_expr()?;
        }

        return Ok(self.nodes.clone());
    }

    fn parse_expr(&mut self) -> Result<(), InkErr> {
        self.idx = self.tokens.len();

        self.nodes.extend(vec![Node::UnaryExpr {
            op: TokKind::NegOp,
            arg: Box::new(Node::NumberLiteral(0.42)),
        }]);

        return Ok(());
    }
}
