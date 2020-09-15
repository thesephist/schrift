use crate::err::InkErr;
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
            Node::BinaryExpr { op, left, right } => {
                let left_reg = self.generate_node(&left)?;
                let right_reg = self.generate_node(&right)?;
                let dest = self.iota();
                self.code.push(Inst {
                    dest,
                    // TODO: make this not always an add
                    op: Op::Add(left_reg, right_reg),
                });
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
                let decl_reg = 0; // TODO: load from local declaration register
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
