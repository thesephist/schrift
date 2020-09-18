use std::fmt;
use std::sync::Arc;

use crate::err::InkErr;
use crate::lex::TokKind;
use crate::parse::Node;
use crate::runtime;

use std::collections::HashMap;

pub type Reg = usize;

pub type NativeFn = fn(Vec<Val>) -> Result<Val, InkErr>;

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum Val {
    Empty,
    Number(f64),
    Str(Vec<u8>),
    Bool(bool),
    Null,
    Comp(HashMap<Vec<u8>, Val>),
    Func(usize), // usize is Block index in Vec<Block>
    NativeFunc(NativeFn),

    // Val::Escaped(Arc<Val>) is a proxy value placed in registers to tell the VM that the register
    // value has been moved to the VM's heap.
    //
    // At compile time:
    // ===============
    //
    // When a variable in scope A register R is determined to have escaped by a closure with scope
    // B (or a composite), the compiler makes these changes:
    //
    // 1. In Block A, add instruction [@R ESCAPE] which tells the VM to move the value to the VM
    //    heap
    // 2. Add a reference (TBD) to Block B's Block::bind vector that will runtime-reference
    //    register @R in A.
    // 2. In Block B, add instruction [@? LOAD_ESC N] when loading the closed-over variable, which
    //    will pull from the runtime-created vec of heap pointers (Vec::Escaped's).
    //
    // At runtime:
    // ===========
    //
    // When the VM LOAD_CONST's a function literal:
    //
    // 1. If the Val::Func's block has any closed-over variable registers in Block::bind, /clone/
    //    the Val::Func and add to it the runtime-determined Vec::Escaped's sitting in those
    //    registers. This produces a new "function object" which is the closure closing over
    //    runtime values sitting on the VM heap.
    //
    // When the VM CALL's a Val::Func:
    //
    // 1. If the Val::Func has any heap pointers in its heap pointer (closed-over variables)
    //    vector, make those Val::Escaped's (heap pointers) available in the vm::Frame in a
    //    predictable way to the frame's bytecode.
    Escaped(Arc<Val>),
}

#[allow(unused)]
impl Val {
    pub fn eq(&self, other: &Val) -> bool {
        match &self {
            Val::Empty => true,
            _ => false,
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum Op {
    Nop,

    Mov(Reg),
    Escape,

    LoadConst(usize),
    LoadBind(usize),
    LoadEsc(usize),

    Call(Reg, Vec<Reg>),
    CallIfEq(Reg, Reg, Reg, usize),

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

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Op::Nop => write!(f, "NOP"),
            Op::Mov(reg) => write!(f, "= @{}", reg),
            Op::Escape => write!(f, "ESCAPE"),
            Op::LoadConst(idx) => write!(f, "LOAD_CONST {}", idx),
            Op::LoadBind(idx) => write!(f, "LOAD_BIND {}", idx),
            Op::LoadEsc(idx) => write!(f, "LOAD_ESC {}", idx),
            Op::Call(reg, args) => write!(
                f,
                "CALL @{}, [{:?}]",
                reg,
                args.iter().map(|r| format!("@{}", r)).collect::<String>()
            ),
            Op::CallIfEq(reg, a, b, jump_by) => {
                write!(f, "CALL @{}, @{} @{}, {}", reg, a, b, jump_by)
            }
            Op::MakeComp => write!(f, "MAKE_COMP"),
            Op::SetComp(reg, k, v) => write!(f, "SET_COMP @{}, @{} @{}", reg, k, v),
            Op::GetComp(reg, k) => write!(f, "SET_COMP @{}, @{}", reg, k),
            Op::Neg(reg) => write!(f, "~ @{}", reg),
            Op::Add(a, b) => write!(f, "@{} + @{}", a, b),
            Op::Sub(a, b) => write!(f, "@{} - @{}", a, b),
            Op::Mul(a, b) => write!(f, "@{} * @{}", a, b),
            Op::Div(a, b) => write!(f, "@{} / @{}", a, b),
            Op::Mod(a, b) => write!(f, "@{} % @{}", a, b),
            Op::Gtr(a, b) => write!(f, "@{} > @{}", a, b),
            Op::Lss(a, b) => write!(f, "@{} < @{}", a, b),
            Op::Eql(a, b) => write!(f, "@{} = @{}", a, b),
            Op::And(a, b) => write!(f, "@{} & @{}", a, b),
            Op::Or(a, b) => write!(f, "@{} | @{}", a, b),
            Op::Xor(a, b) => write!(f, "@{} ^ @{}", a, b),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Inst {
    pub dest: Reg,
    pub op: Op,
}

impl fmt::Display for Inst {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "@{}\t{}", self.dest, self.op)
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub slots: usize,
    pub consts: Vec<Val>,
    pub binds: Vec<Reg>,
    pub code: Vec<Inst>,

    // integer counter to label autoincremented
    // pseudo-register allocations.
    iota: usize,
    scope: HashMap<String, Reg>,
    parent: Option<Box<Block>>,
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "consts: {:?}", self.consts)?;
        writeln!(f, "binds: {:?}", self.binds)?;
        for inst in self.code.iter() {
            writeln!(f, "  {}", inst)?;
        }
        write!(f, "")
    }
}

impl Block {
    fn new() -> Block {
        return Block {
            slots: 0,
            consts: Vec::new(),
            binds: Vec::new(),
            code: vec![],
            iota: 0,
            scope: HashMap::new(),
            parent: None,
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

    fn scope_get(&mut self, name: &String) -> Option<&Reg> {
        // when a get() needs to cross block boundaries, move the register
        // to self.binds.
        return match self.scope.get(name) {
            Some(reg) => Some(reg),
            None => match &mut self.parent {
                // TODO: need some way to mark that this is a closure/bound-get
                // of an escaped var and not a local variable to the block/frame.
                // We do this by addding the ESCAPE instruction in the parent scope
                // against the variable's register.
                Some(parent) => match parent.scope_get(name) {
                    Some(reg) => {
                        self.binds.push(reg.clone());
                        Some(reg)
                    }
                    None => None,
                },
                None => None,
            },
        };
    }

    fn scope_insert(&mut self, name: String, reg: Reg) -> Option<Reg> {
        return self.scope.insert(name, reg);
    }

    fn from_nodes<F>(nodes: Vec<Node>, push_block: &mut F) -> Result<Block, InkErr>
    where
        F: FnMut(Block) -> usize,
    {
        let mut block = Block::new();
        block.generate_nodes(nodes, push_block)?;
        return Ok(block);
    }

    fn generate_nodes<F>(&mut self, nodes: Vec<Node>, push_block: &mut F) -> Result<(), InkErr>
    where
        F: FnMut(Block) -> usize,
    {
        for node in nodes.iter() {
            self.generate_node(&node, push_block)?;
        }
        self.slots = self.iota;
        return Ok(());
    }

    // returns the register at which the result of evaluating `node`
    // is stored, after executing all generated code for the given node.
    fn generate_node<F>(&mut self, node: &Node, push_block: &mut F) -> Result<Reg, InkErr>
    where
        F: FnMut(Block) -> usize,
    {
        let result_reg = match node {
            Node::UnaryExpr { op: _, arg } => {
                let arg_reg = self.generate_node(&arg, push_block)?;
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
                let right_reg = self.generate_node(&define_right, push_block)?;

                match *define_left.clone() {
                    Node::BinaryExpr {
                        op: TokKind::AccessorOp,
                        left: comp_left,
                        right: comp_right,
                    } => {
                        let comp_left_reg = self.generate_node(&comp_left, push_block)?;
                        let comp_right_reg = self.generate_node(&comp_right, push_block)?;

                        let dest = self.iota();
                        self.code.push(Inst {
                            dest,
                            op: Op::SetComp(comp_left_reg, comp_right_reg, right_reg),
                        });
                        right_reg
                    }
                    Node::Ident(name) => {
                        let dest = self.iota();
                        self.scope_insert(name.clone(), dest);
                        self.code.push(Inst {
                            dest,
                            op: Op::Mov(right_reg),
                        });
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
                let left_reg = self.generate_node(&left, push_block)?;
                let right_reg = self.generate_node(&right, push_block)?;
                let dest = self.iota();
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
                let func_reg = self.generate_node(&func, push_block)?;
                let mut arg_regs = Vec::new();
                for arg in args.iter() {
                    arg_regs.push(self.generate_node(arg, push_block)?);
                }
                let dest = self.iota();
                self.code.push(Inst {
                    dest,
                    op: Op::Call(func_reg, arg_regs),
                });
                dest
            }
            Node::MatchClause {
                target: _target,
                expr: _expr,
            } => {
                // TODO: must produce block per clause
                self.iota()
            }
            Node::MatchExpr {
                cond: _cond,
                clauses: _clauses,
            } => {
                // TODO: must produce block per clause
                self.iota()
            }
            Node::ExprList(exprs) => {
                let exprlist_block = Block::from_nodes(exprs.clone(), push_block)?;
                let block_idx = push_block(exprlist_block);

                let closure_dest = self.iota();
                let const_dest = self.push_const(Val::Func(block_idx));
                self.code.push(Inst {
                    dest: closure_dest,
                    op: Op::LoadConst(const_dest),
                });
                let call_dest = self.iota();
                self.code.push(Inst {
                    dest: call_dest,
                    op: Op::Call(closure_dest, Vec::new()),
                });
                call_dest
            }
            Node::EmptyIdent => {
                let dest = self.iota();
                self.code.push(Inst { dest, op: Op::Nop });
                dest
            }
            Node::Ident(name) => match self.scope_get(name) {
                Some(reg) => reg.clone(),
                None => {
                    let dest = self.iota();
                    let const_dest: Reg;
                    const_dest = match name.as_str() {
                        "out" => self.push_const(Val::NativeFunc(runtime::builtin_out)),
                        "char" => self.push_const(Val::NativeFunc(runtime::builtin_char)),
                        "string" => self.push_const(Val::NativeFunc(runtime::builtin_string)),
                        _ => {
                            println!("Could not find variable {:?} in current scope", name);
                            return Err(InkErr::UndefinedVariable);
                        }
                    };
                    self.code.push(Inst {
                        dest,
                        op: Op::LoadConst(const_dest),
                    });
                    dest
                }
            },
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
            Node::ObjectEntry {
                key: _key,
                val: _val,
            } => {
                // TODO: generate object entry insertion code
                self.iota()
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
                            let key_reg = self.generate_node(key, push_block)?;
                            let val_reg = self.generate_node(val, push_block)?;
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

                    let item_reg = self.generate_node(item, push_block)?;
                    let item_dest = self.iota();
                    self.code.push(Inst {
                        dest: item_dest,
                        op: Op::SetComp(dest, index_dest, item_reg),
                    });
                }
                dest
            }
            Node::FnLiteral { args, body } => {
                let mut func_block = Block::new();
                for arg in args.iter() {
                    match arg {
                        Node::Ident(name) => {
                            let arg_reg = func_block.iota();
                            func_block.scope.insert(name.clone(), arg_reg);
                        }
                        _ => (),
                    }
                }
                match &**body {
                    Node::ExprList(exprs) => {
                        func_block.generate_nodes(exprs.to_vec(), push_block)?
                    }
                    _ => func_block.generate_nodes(vec![*body.clone()], push_block)?,
                }

                let block_idx = push_block(func_block);

                let dest = self.iota();
                let const_dest = self.push_const(Val::Func(block_idx));
                self.code.push(Inst {
                    dest,
                    op: Op::LoadConst(const_dest),
                });
                dest
            }
        };

        return Ok(result_reg);
    }
}

pub fn generate(nodes: Vec<Node>) -> Result<Vec<Block>, InkErr> {
    let mut prog = Vec::<Block>::new();
    let main_block = Block::from_nodes(nodes, &mut |block| {
        prog.push(block);
        return prog.len();
    })?;

    // ensure main loop is first
    let mut main_prog = vec![main_block];
    main_prog.append(&mut prog);

    return Ok(main_prog);
}
