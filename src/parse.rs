use crate::err::InkErr;
use crate::lex::{Tok, TokKind};

#[allow(dead_code)]
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

impl Tok<'_> {
    fn priority(&self) -> i32 {
        // higher == greater priority
        match self.kind {
            TokKind::AccessorOp => 100,
            TokKind::ModOp => 80,
            TokKind::MulOp | TokKind::DivOp => 50,
            TokKind::AddOp | TokKind::SubOp => 40,
            TokKind::GtOp | TokKind::LtOp | TokKind::EqOp => 30,
            TokKind::AndOp => 20,
            TokKind::XorOp => 15,
            TokKind::OrOp => 10,
            TokKind::DefineOp => 0,
            _ => -1,
        }
    }

    fn is_binary(&self) -> bool {
        return self.priority() > 0;
    }
}

type ParseResult = Result<Vec<Node>, InkErr>;

pub fn parse(tokens: Vec<Tok>) -> ParseResult {
    let tokens_without_comments: Vec<Tok> = tokens
        .into_iter()
        .filter(|tok| match tok.kind {
            TokKind::Comment(_) => false,
            _ => true,
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

    fn guard_eof(&self) -> Result<(), InkErr> {
        if self.idx > self.tokens.len() {
            return Err(InkErr::UnexpectedEOF);
        } else {
            return Ok(());
        }
    }

    fn parse(&mut self) -> ParseResult {
        while self.idx < self.tokens.len() {
            let node = self.parse_expr()?;
            self.nodes.push(node);
        }

        return Ok(self.nodes.clone());
    }

    fn consume_dangling_separator(&mut self) {
        if self.idx < self.tokens.len() && self.tokens[self.idx].kind == TokKind::Separator {
            self.idx += 1;
        }
    }

    fn parse_expr(&mut self) -> Result<Node, InkErr> {
        let atom = self.parse_atom()?;

        self.guard_eof()?;
        let next = &self.tokens[self.idx];
        self.idx += 1;

        match next.kind {
            // consuming dangling separator
            TokKind::Separator => return Ok(atom),
            TokKind::KeyValueSeparator | TokKind::RParen => {
                // These belong to the parent atom that contains this expression, so return without
                // consuming token (idx - 1)
                self.idx -= 1;
                return Ok(atom);
            }
            _ if next.is_binary() => {
                let next_tok = next.clone();
                let bin_expr = self.parse_binary_expr(atom, next_tok, -1)?;

                // BinExpr are sometimes followed by a match
                if self.idx < self.tokens.len() && self.tokens[self.idx].kind == TokKind::MatchColon
                {
                    self.idx += 1; // MatchColon

                    let clauses = self.parse_match_body()?;
                    self.consume_dangling_separator();

                    return Ok(Node::MatchExpr {
                        cond: Box::new(bin_expr),
                        clauses: clauses,
                    });
                }

                self.consume_dangling_separator();
                return Ok(bin_expr);
            }
            TokKind::MatchColon => {
                let clauses = self.parse_match_body()?;
                self.consume_dangling_separator();

                return Ok(Node::MatchExpr {
                    cond: Box::new(atom),
                    clauses: clauses,
                });
            }
            _ => Err(InkErr::UnexpectedToken),
        }
    }

    fn parse_binary_expr(
        &mut self,
        left: Node,
        op: Tok,
        prev_priority: i32,
    ) -> Result<Node, InkErr> {
        let right = self.parse_atom()?;

        let mut ops = vec![op];
        let mut nodes = vec![left, right];

        // build up a list of binary operations, with tree nodes
        // where there are higher-priority binary ops
        while self.tokens.len() > self.idx && self.tokens[self.idx].is_binary() {
            if prev_priority >= self.tokens[self.idx].priority() {
                // Priority is lower than the calling functiono's last op,
                // so return control to the parent binary op
                break;
            } else if ops.last().unwrap().priority() >= self.tokens[self.idx].priority() {
                // Priority is lower than the previous op but higher than parent,
                // so it's ok to be left-heavy in this tree
                ops.push(self.tokens[self.idx].clone());
                self.idx += 1;

                self.guard_eof()?;

                let right = self.parse_atom()?;
                nodes.push(right);
            } else {
                // Priority is higher than previous ops, so
                // make it a right-heavy tree branch
                self.idx += 1;
                self.guard_eof()?;

                let subtree = self.parse_binary_expr(
                    nodes.pop().unwrap(),
                    self.tokens[self.idx].clone(),
                    ops.last().unwrap().priority(),
                )?;

                nodes.push(subtree);
                self.idx += 1;
            }
        }

        // ops, nodes -> left-biased binary expression tree
        let mut tree = nodes[0].clone();
        let mut nodes_slice = &nodes[1..];
        let mut ops_slice = &ops[..];

        while ops_slice.len() > 0 {
            tree = Node::BinaryExpr {
                op: ops_slice[0].clone().kind,
                left: Box::new(tree.clone()),
                right: Box::new(nodes_slice[0].clone()),
            };

            ops_slice = &ops_slice[1..];
            nodes_slice = &nodes_slice[1..];
        }

        return Ok(tree);
    }

    // parse_atom returns its result instead of pushing to self.nodes
    fn parse_atom(&mut self) -> Result<Node, InkErr> {
        self.guard_eof()?;

        let tok = &self.tokens[self.idx].clone();
        self.idx += 1;

        if tok.kind == TokKind::NegOp {
            let atom = self.parse_atom()?;
            return Ok(Node::UnaryExpr {
                op: tok.kind.clone(),
                arg: Box::new(atom),
            });
        }

        self.guard_eof()?;

        match tok.kind.clone() {
            TokKind::NumberLiteral(num) => return Ok(Node::NumberLiteral(num)),
            TokKind::StringLiteral(s) => return Ok(Node::StringLiteral(s)),
            TokKind::TrueLiteral => return Ok(Node::BooleanLiteral(true)),
            TokKind::FalseLiteral => return Ok(Node::BooleanLiteral(false)),
            TokKind::Ident(_) => {
                return Err(InkErr::Unimplemented);
            }
            TokKind::EmptyIdent => {
                return Err(InkErr::Unimplemented);
            }
            TokKind::LParen => {
                return Err(InkErr::Unimplemented);
            }
            TokKind::LBrace => {
                return Err(InkErr::Unimplemented);
            }
            TokKind::LBracket => {
                return Err(InkErr::Unimplemented);
            }
            _ => (),
        }

        // TODO: parse FnCall

        return Err(InkErr::Unimplemented);
    }

    fn parse_match_body(&self) -> Result<Vec<Node>, InkErr> {
        return Err(InkErr::Unimplemented);
    }
}
