use crate::err::InkErr;
use crate::lex::TokKind;
use crate::parse::Node;

use std::collections::HashMap;

type Reg = usize;

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum Val {
    Empty,
    Number(f64),
    Str(Vec<u8>),
    Bool(bool),
    Null,
    Comp(HashMap<Vec<u8>, Val>),
    Func(Block),
}

impl Val {
    fn to_ink_string(&self) -> String {
        match &self {
            // TODO: implement
            _ => String::from("(unimplemented)"),
        }
    }

    fn eq(&self, other: &Val) -> bool {
        match &self {
            // TODO: implement
            _ => false,
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum Op {
    Nop,

    Mov(Reg),

    LoadArg(usize),
    LoadConst(usize),
    LoadBind(usize),
    LoadBlock(Reg),

    Call(Reg),
    CallIfEq(Reg, Reg, Reg),

    MakeComp,
    SetComp(Reg, Reg, Reg),
    GetComp(Reg, Reg),

    Neg(Reg),
    Add(Reg, Reg),
    Sub(Reg, Reg),
    Mul(Reg, Reg),
    Div(Reg, Reg),
    Mod(Reg, Reg),

    Gtr(Reg, Reg),
    Lss(Reg, Reg),
    Eql(Reg, Reg),

    And(Reg, Reg),
    Or(Reg, Reg),
    Xor(Reg, Reg),
}

#[derive(Debug, Clone)]
pub struct Inst {
    dest: Reg,
    op: Op,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub slots: usize,
    pub consts: Vec<Val>,
    pub binds: Vec<Reg>,
    pub ret: Reg,
    pub code: Vec<Inst>,

    // integer counter to label
    // pseudo-register allocations.
    iota: usize,
}

impl Block {
    fn new() -> Block {
        return Block {
            slots: 0,
            consts: Vec::new(),
            binds: Vec::new(),
            ret: 0,
            code: vec![],
            iota: 0,
        };
    }

    fn iota(&mut self) -> Reg {
        let last = self.iota;
        self.iota += 1;
        return last;
    }

    fn push_const(&mut self, val: Val) -> Reg {
        self.consts.push(val);
        return self.consts.len() - 1;
    }

    // returns the register at which the result of evaluating `node`
    // is stored, after executing all generated code for the given node.
    fn generate_node(&mut self, node: &Node) -> Result<Reg, InkErr> {
        let result_reg = match node {
            Node::UnaryExpr { op, arg } => {
                let arg_reg = self.generate_node(&arg)?;
                let dest = self.iota();
                self.code.push(Inst {
                    dest,
                    op: Op::Neg(arg_reg),
                });
                dest
            }
            Node::BinaryExpr {
                op: TokKind::DefineOp,
                left: define_left,
                right: define_right,
            } => {
                let right_reg = self.generate_node(&define_right)?;

                match *(define_left.clone()) {
                    Node::BinaryExpr {
                        op: TokKind::AccessorOp,
                        left: comp_left,
                        right: comp_right,
                    } => {
                        let dest = self.iota();
                        self.code.push(Inst {
                            dest,
                            // TODO: Op::SetComp for comp register, key register, and right_reg;
                            op: Op::Nop,
                        });
                        right_reg
                    }
                    Node::Ident(_) => {
                        let dest = self.iota();
                        // TODO: return register allocated by variable declaration
                        dest
                    }
                    Node::EmptyIdent => right_reg,
                    _ => {
                        println!("Invalid assignment expression: {:?}", node);
                        return Err(InkErr::InvalidAssignment);
                    }
                }
            }
            Node::BinaryExpr { op, left, right } => {
                let left_reg = self.generate_node(&left)?;
                let right_reg = self.generate_node(&right)?;
                let dest = self.iota();
                // TODO: make this not always an add
                match op {
                    TokKind::AddOp => self.code.push(Inst {
                        dest,
                        op: Op::Add(left_reg, right_reg),
                    }),
                    TokKind::SubOp => self.code.push(Inst {
                        dest,
                        op: Op::Sub(left_reg, right_reg),
                    }),
                    TokKind::MulOp => self.code.push(Inst {
                        dest,
                        op: Op::Mul(left_reg, right_reg),
                    }),
                    TokKind::DivOp => self.code.push(Inst {
                        dest,
                        op: Op::Div(left_reg, right_reg),
                    }),
                    TokKind::ModOp => self.code.push(Inst {
                        dest,
                        op: Op::Mod(left_reg, right_reg),
                    }),
                    TokKind::GtOp => self.code.push(Inst {
                        dest,
                        op: Op::Gtr(left_reg, right_reg),
                    }),
                    TokKind::LtOp => self.code.push(Inst {
                        dest,
                        op: Op::Lss(left_reg, right_reg),
                    }),
                    TokKind::EqOp => self.code.push(Inst {
                        dest,
                        op: Op::Eql(left_reg, right_reg),
                    }),
                    TokKind::AndOp => self.code.push(Inst {
                        dest,
                        op: Op::And(left_reg, right_reg),
                    }),
                    TokKind::OrOp => self.code.push(Inst {
                        dest,
                        op: Op::Or(left_reg, right_reg),
                    }),
                    TokKind::XorOp => self.code.push(Inst {
                        dest,
                        op: Op::Xor(left_reg, right_reg),
                    }),
                    TokKind::AccessorOp => self.code.push(Inst {
                        dest,
                        op: Op::GetComp(left_reg, right_reg),
                    }),
                    _ => {
                        println!("Cannot compile binary op {:?}", op);
                        return Err(InkErr::Unimplemented);
                    }
                }
                dest
            }
            Node::FnCall { func, args } => {
                let func_reg = self.generate_node(&func)?;
                // TODO: how do we encode fn args in bytecode?
                let dest = self.iota();
                self.code.push(Inst {
                    dest,
                    op: Op::Call(func_reg),
                });
                dest
            }
            Node::MatchExpr { cond, clauses } => {
                // TODO: must produce block per clause
                self.iota()
            }
            Node::ExprList(exprs) => {
                // TODO: must produce another block!
                self.iota()
            }
            Node::EmptyIdent => {
                let dest = self.iota();
                self.code.push(Inst { dest, op: Op::Nop });
                dest
            }
            Node::Ident(name) => {
                // TODO: load from local declaration register annotated on the Node::Ident
                let decl_reg = 0;

                let dest = self.iota();
                self.code.push(Inst {
                    dest,
                    op: Op::Mov(decl_reg),
                });
                dest
            }
            Node::NumberLiteral(n) => {
                let dest = self.iota();
                let const_dest = self.push_const(Val::Number(n.clone()));
                self.code.push(Inst {
                    dest,
                    op: Op::LoadConst(const_dest),
                });
                dest
            }
            Node::StringLiteral(s) => {
                let dest = self.iota();
                let const_dest = self.push_const(Val::Str(s.clone().into_bytes()));
                self.code.push(Inst {
                    dest,
                    op: Op::LoadConst(const_dest),
                });
                dest
            }
            Node::BooleanLiteral(b) => {
                let dest = self.iota();
                let const_dest = self.push_const(Val::Bool(b.clone()));
                self.code.push(Inst {
                    dest,
                    op: Op::LoadConst(const_dest),
                });
                dest
            }
            Node::ObjectLiteral(entries) => {
                let dest = self.iota();
                self.code.push(Inst {
                    dest,
                    op: Op::MakeComp,
                });
                for entry in entries.iter() {
                    match entry {
                        Node::ObjectEntry { key, val } => {
                            let key_reg = self.generate_node(key)?;
                            let val_reg = self.generate_node(val)?;
                            let entry_dest = self.iota();
                            self.code.push(Inst {
                                dest: entry_dest,
                                op: Op::SetComp(dest, key_reg, val_reg),
                            });
                        }
                        _ => panic!("unreachable!"),
                    }
                }
                dest
            }
            Node::ListLiteral(items) => {
                let dest = self.iota();
                self.code.push(Inst {
                    dest,
                    op: Op::MakeComp,
                });
                for (i, item) in items.iter().enumerate() {
                    let index_dest = self.iota();
                    let index_reg = self.push_const(Val::Number(i as f64));
                    self.code.push(Inst {
                        dest: index_dest,
                        op: Op::LoadConst(index_reg),
                    });

                    let item_reg = self.generate_node(item)?;
                    let item_dest = self.iota();
                    self.code.push(Inst {
                        dest: item_dest,
                        op: Op::SetComp(dest, index_dest, item_reg),
                    });
                }
                dest
            }
            Node::FnLiteral { args, body } => {
                // TODO: must produce another block!
                self.iota()
            }
            _ => {
                let dest = self.iota();
                self.code.push(Inst { dest, op: Op::Nop });
                dest
            }
        };

        return Ok(result_reg);
    }
}

pub fn generate(nodes: Vec<Node>) -> Result<Vec<Block>, InkErr> {
    let mut main_block = Block::new();

    for node in nodes.iter() {
        main_block.generate_node(&node)?;
    }

    let mut program = Vec::<Block>::new();
    program.push(main_block);
    return Ok(program);
}
