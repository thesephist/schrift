use crate::err;
use crate::lex::{Tok, TokKind};

#[derive(Debug)]
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

pub fn parse(tokens: Vec<Tok>) -> Result<Vec<Node>, err::InkErr> {
    let tokens_without_comments: Vec<Tok> = tokens
        .into_iter()
        .filter(|tok| match tok.kind {
            TokKind::Comment(_) => true,
            _ => false,
        })
        .collect();

    return Ok(vec![Node::UnaryExpr {
        op: TokKind::NegOp,
        arg: Box::new(Node::NumberLiteral(0.42)),
    }]);
}
