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
            TokKind::DefineOp => 5,
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

                nodes.push(self.parse_atom()?);
            } else {
                // Priority is higher than previous ops, so
                // make it a right-heavy tree branch
                self.guard_eof()?;

                let next_op = self.tokens[self.idx].clone();
                self.idx += 1;
                let subtree = self.parse_binary_expr(
                    nodes.pop().unwrap(),
                    next_op,
                    ops.last().unwrap().priority(),
                )?;

                nodes.push(subtree);
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

        let tok = self.tokens[self.idx].clone();
        self.idx += 1;

        if tok.kind == TokKind::NegOp {
            let atom = self.parse_atom()?;
            return Ok(Node::UnaryExpr {
                op: tok.kind.clone(),
                arg: Box::new(atom),
            });
        }

        self.guard_eof()?;

        let mut atom: Node;
        match tok.kind.clone() {
            TokKind::NumberLiteral(num) => return Ok(Node::NumberLiteral(num)),
            TokKind::StringLiteral(s) => return Ok(Node::StringLiteral(s)),
            TokKind::TrueLiteral => return Ok(Node::BooleanLiteral(true)),
            TokKind::FalseLiteral => return Ok(Node::BooleanLiteral(false)),
            TokKind::Ident(s) => {
                if self.tokens[self.idx].kind == TokKind::FunctionArrow {
                    self.idx -= 1;
                    atom = self.parse_fn_literal_monadic()?;

                    // parse_atom should not consume trailing Separators, but
                    // parse_fn_literal does because it ends with expressions.
                    // so we backtrack one token.
                    self.idx -= 1;
                } else {
                    atom = Node::Ident(s)
                }
                // fallthrough
            }
            TokKind::EmptyIdent => {
                if self.tokens[self.idx].kind == TokKind::FunctionArrow {
                    self.idx -= 1;
                    atom = self.parse_fn_literal_monadic()?;

                    // parse_atom should not consume trailing Separators, but
                    // parse_fn_literal does because it ends with expressions.
                    // so we backtrack one token.
                    self.idx -= 1;
                    return Ok(atom);
                }
                return Ok(Node::EmptyIdent);
            }
            TokKind::LParen => {
                // expression list, or argument list for a function literal
                let mut exprs = Vec::<Node>::new();
                let lparen_idx = self.idx - 1;
                while self.tokens[self.idx].kind != TokKind::RParen {
                    exprs.push(self.parse_expr()?);
                    self.guard_eof()?;
                }
                self.idx += 1; // RParen
                self.guard_eof()?;

                if self.tokens[self.idx].kind == TokKind::FunctionArrow {
                    self.idx = lparen_idx;
                    atom = self.parse_fn_literal_variadic()?;

                    // parse_atom should not consume trailing Separators, but
                    // parse_fn_literal does because it ends with expressions.
                    // so we backtrack one token.
                    self.idx -= 1;
                } else {
                    atom = Node::ExprList(exprs);
                }
                // fallthrough
            }
            TokKind::LBrace => {
                let mut entries = Vec::<Node>::new();
                while self.tokens[self.idx].kind != TokKind::RBrace {
                    let key_expr = self.parse_expr()?;
                    self.guard_eof()?;

                    if self.tokens[self.idx].kind == TokKind::KeyValueSeparator {
                        self.idx += 1; // KeyValueSeparator
                    } else {
                        return Err(InkErr::ExpectedCompositeValue);
                    }

                    self.guard_eof()?;

                    let val_expr = self.parse_expr()?;

                    // Separator after val_expr is consumed by parse_expr
                    entries.push(Node::ObjectEntry {
                        key: Box::new(key_expr),
                        val: Box::new(val_expr),
                    });

                    self.guard_eof()?;
                }
                self.idx += 1; // RBrace

                return Ok(Node::ObjectLiteral(entries));
            }
            TokKind::LBracket => {
                let mut items = Vec::<Node>::new();
                while self.tokens[self.idx].kind != TokKind::RBracket {
                    items.push(self.parse_expr()?);
                    self.guard_eof()?;
                }
                self.idx += 1; // RBracket

                return Ok(Node::ListLiteral(items));
            }
            _ => return Err(InkErr::UnexpectedToken),
        }

        // bounds check here because parse_expr may have consumed all tokens before this
        while self.idx < self.tokens.len() && self.tokens[self.idx].kind == TokKind::LParen {
            atom = self.parse_fn_call(atom)?;
            self.guard_eof()?;
        }

        return Ok(atom);
    }

    fn parse_match_body(&mut self) -> Result<Vec<Node>, InkErr> {
        self.idx += 1; // LBrace
        let mut clauses = Vec::<Node>::new();

        self.guard_eof()?;

        while self.tokens[self.idx].kind != TokKind::RBrace {
            clauses.push(self.parse_match_clause()?);
            self.guard_eof()?;
        }
        self.idx += 1; // RBrace

        return Ok(clauses);
    }

    fn parse_match_clause(&mut self) -> Result<Node, InkErr> {
        let atom = self.parse_atom()?;
        self.guard_eof()?;

        if self.tokens[self.idx].kind != TokKind::CaseArrow {
            return Err(InkErr::ExpectedMatchCaseArrow);
        }
        self.idx += 1; // CaseArrow
        self.guard_eof()?;

        let expr = self.parse_expr()?;

        return Ok(Node::MatchClause {
            target: Box::new(atom),
            expr: Box::new(expr),
        });
    }

    fn parse_fn_literal_monadic(&mut self) -> Result<Node, InkErr> {
        let mut args = Vec::<Node>::new();

        let kind = &self.tokens[self.idx].kind;
        match kind {
            TokKind::Ident(s) => args.push(Node::Ident(s.clone())),
            TokKind::EmptyIdent => args.push(Node::EmptyIdent),
            _ => return Err(InkErr::UnexpectedArgument),
        }
        self.idx += 1; // [Empty]Ident
        self.guard_eof()?;

        if self.tokens[self.idx].kind != TokKind::FunctionArrow {
            return Err(InkErr::UnexpectedToken);
        }
        self.idx += 1; // FunctionArrow

        let body = self.parse_expr()?;

        return Ok(Node::FnLiteral {
            args: args,
            body: Box::new(body),
        });
    }

    fn parse_fn_literal_variadic(&mut self) -> Result<Node, InkErr> {
        self.idx += 1; // LParen

        let mut args = Vec::<Node>::new();
        while self.tokens[self.idx].kind != TokKind::RParen {
            let kind = &self.tokens[self.idx].kind;
            match kind {
                TokKind::Ident(s) => args.push(Node::Ident(s.clone())),
                TokKind::EmptyIdent => args.push(Node::EmptyIdent),
                _ => return Err(InkErr::UnexpectedArgument),
            }
            self.idx += 1; // [Empty]Ident
            self.guard_eof()?;

            if self.tokens[self.idx].kind != TokKind::Separator {
                return Err(InkErr::UnexpectedToken);
            }

            self.idx += 1; // Separator

            // guard_eof not necessary here because a file always ends with Separator
        }
        self.guard_eof()?;

        if self.tokens[self.idx].kind != TokKind::RParen {
            return Err(InkErr::UnexpectedToken);
        }
        self.idx += 1; // RParen
        self.guard_eof()?;

        if self.tokens[self.idx].kind != TokKind::FunctionArrow {
            return Err(InkErr::UnexpectedToken);
        }
        self.idx += 1; // FunctionArrow

        let body = self.parse_expr()?;

        return Ok(Node::FnLiteral {
            args: args,
            body: Box::new(body),
        });
    }

    fn parse_fn_call(&mut self, func: Node) -> Result<Node, InkErr> {
        self.idx += 1; // LParen
        self.guard_eof()?;

        let mut args = Vec::<Node>::new();

        while self.tokens[self.idx].kind != TokKind::RParen {
            args.push(self.parse_expr()?);
            self.guard_eof()?;
        }
        self.idx += 1; // RParen

        return Ok(Node::FnCall {
            func: Box::new(func),
            args: args,
        });
    }
}
