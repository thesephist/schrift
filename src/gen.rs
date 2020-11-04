use std::fmt;

use crate::err::InkErr;
use crate::lex::TokKind;
use crate::parse::Node;
use crate::runtime;
use crate::val::{NativeFn, Val};

use std::collections::HashMap;

pub type Reg = usize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Nop,

    Mov(Reg),
    Escape(Reg),

    LoadConst(usize),
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
            Op::Escape(reg) => write!(f, "ESCAPE @{}", reg),
            Op::Mov(reg) => write!(f, "MOV @{}", reg),
            Op::LoadConst(idx) => write!(f, "LOAD_CONST {}", idx),
            Op::LoadEsc(idx) => write!(f, "LOAD_ESC {}", idx),
            Op::Call(reg, args) => write!(
                f,
                "CALL @{}, [{:?}]",
                reg,
                args.iter().map(|r| format!("@{}", r)).collect::<String>()
            ),
            Op::CallIfEq(reg, a, b, jump_by) => {
                write!(f, "CALL_IF_EQ @{}, @{} == @{}, {}", reg, a, b, jump_by)
            }
            Op::MakeComp => write!(f, "MAKE_COMP"),
            Op::SetComp(reg, k, v) => write!(f, "SET_COMP @{}, @{} @{}", reg, k, v),
            Op::GetComp(reg, k) => write!(f, "GET_COMP @{}, @{}", reg, k),
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
struct ScopeRecord {
    reg: Reg,
    from_current_scope: bool,
    forward_decl: bool,
    escaped: bool,
}

struct ScopeStack {
    scopes: Vec<HashMap<String, ScopeRecord>>,
}

impl ScopeStack {
    fn new() -> ScopeStack {
        return ScopeStack {
            scopes: vec![HashMap::new()],
        };
    }

    fn push(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop(&mut self) {
        self.scopes.pop();
    }

    fn last(&self) -> &HashMap<String, ScopeRecord> {
        return self.scopes.last().unwrap();
    }

    fn get(&mut self, name: &String) -> Option<ScopeRecord> {
        for (i, scope) in self.scopes.iter_mut().rev().enumerate() {
            match scope.get_mut(name) {
                Some(rec) if rec.from_current_scope => {
                    let escaped = i > 0;
                    if escaped {
                        rec.escaped = true;
                    }
                    return Some(ScopeRecord {
                        reg: rec.reg,
                        from_current_scope: i == 0,
                        forward_decl: rec.forward_decl,
                        escaped: rec.escaped,
                    });
                }
                _ => {
                    scope.insert(
                        name.to_string(),
                        ScopeRecord {
                            reg: 0, // dummy reg
                            from_current_scope: false,
                            forward_decl: false,
                            escaped: true,
                        },
                    );
                }
            };
        }

        return None;
    }

    fn insert(&mut self, name: String, reg: Reg) {
        self.scopes.last_mut().unwrap().insert(
            name,
            ScopeRecord {
                reg,
                from_current_scope: true,
                forward_decl: false,
                escaped: false,
            },
        );
    }

    fn forward_declare(&mut self, name: String, reg: Reg) {
        self.scopes.last_mut().unwrap().insert(
            name,
            ScopeRecord {
                reg,
                from_current_scope: true,
                forward_decl: true,
                escaped: false,
            },
        );
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub slots: usize,
    pub consts: Vec<Val>,

    // binds_names is a list of names closed over
    // used to link registers to those names in a parent scope.
    // Escaped names should appear in binds_names and binds in corresponding order.
    pub binds_names: Vec<String>,
    pub binds: Vec<Reg>,
    pub code: Vec<Inst>,

    // integer counter to label autoincremented
    // pseudo-register allocations.
    iota: usize,
    parent: Option<Box<Block>>,
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "consts: [")?;
        for (i, c) in self.consts.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", c)?
        }
        writeln!(f, "]")?;
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
            consts: vec![],
            binds_names: vec![],
            binds: vec![],
            code: vec![],
            iota: 0,
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

    fn from_nodes<F>(
        nodes: Vec<Node>,
        scopes: &mut ScopeStack,
        push_block: &mut F,
    ) -> Result<Block, InkErr>
    where
        F: FnMut(Block) -> usize,
    {
        let mut block = Block::new();
        block.generate_nodes(nodes, scopes, push_block)?;
        return Ok(block);
    }

    fn generate_nodes<F>(
        &mut self,
        nodes: Vec<Node>,
        scopes: &mut ScopeStack,
        push_block: &mut F,
    ) -> Result<(), InkErr>
    where
        F: FnMut(Block) -> usize,
    {
        // hoisted (forward) declarations for this scope
        for node in nodes.iter() {
            if let Node::BinaryExpr {
                op: TokKind::DefineOp,
                left: define_left,
                right: _,
            } = node
            {
                if let Node::Ident(name) = &**define_left {
                    scopes.forward_declare(name.clone(), self.iota());
                }
            }
        }
        for node in nodes.iter() {
            self.generate_node(&node, scopes, push_block)?;
        }
        self.slots = self.iota;
        return Ok(());
    }

    // returns the register at which the result of evaluating `node`
    // is stored, after executing all generated code for the given node.
    fn generate_node<F>(
        &mut self,
        node: &Node,
        mut scopes: &mut ScopeStack,
        push_block: &mut F,
    ) -> Result<Reg, InkErr>
    where
        F: FnMut(Block) -> usize,
    {
        let result_reg = match node {
            Node::UnaryExpr { op: _, arg } => {
                let arg_reg = self.generate_node(&arg, &mut scopes, push_block)?;
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
                let right_reg = self.generate_node(&define_right, &mut scopes, push_block)?;

                match &**define_left {
                    Node::BinaryExpr {
                        op: TokKind::AccessorOp,
                        left: comp_left,
                        right: comp_right,
                    } => {
                        let comp_left_reg =
                            self.generate_node(&comp_left, &mut scopes, push_block)?;
                        let comp_right_reg = if let Node::Ident(name) = &**comp_right {
                            let right_as_str = Node::StringLiteral(name.clone());
                            self.generate_node(&right_as_str, &mut scopes, push_block)?
                        } else {
                            self.generate_node(&comp_right, &mut scopes, push_block)?
                        };

                        let dest = self.iota();
                        self.code.push(Inst {
                            dest,
                            op: Op::SetComp(comp_left_reg, comp_right_reg, right_reg),
                        });
                        comp_left_reg
                    }
                    Node::Ident(name) => match scopes.get(name) {
                        Some(rec) => {
                            self.code.push(Inst {
                                dest: rec.reg,
                                op: Op::Mov(right_reg),
                            });
                            scopes.insert(name.clone(), rec.reg);
                            rec.reg
                        }
                        // We expect all name bindings to be forward-declared
                        // at the top of this scope's codegen.
                        None => {
                            println!(
                                "Could not find forward-declared \"{:?}\" in current scope",
                                name
                            );
                            return Err(InkErr::UndefinedVariable);
                        }
                    },
                    Node::EmptyIdent => right_reg,
                    _ => {
                        println!("Invalid assignment expression: {:?}", node);
                        return Err(InkErr::InvalidAssignment);
                    }
                }
            }
            Node::BinaryExpr {
                op: TokKind::AccessorOp,
                left: access_left,
                right: access_right,
            } => {
                let left_reg = self.generate_node(&access_left, &mut scopes, push_block)?;
                let right_reg = if let Node::Ident(name) = &**access_right {
                    let right_as_str = Node::StringLiteral(name.clone());
                    self.generate_node(&right_as_str, &mut scopes, push_block)?
                } else {
                    self.generate_node(&access_right, &mut scopes, push_block)?
                };
                let dest = self.iota();
                self.code.push(Inst {
                    dest,
                    op: Op::GetComp(left_reg, right_reg),
                });
                dest
            }
            Node::BinaryExpr { op, left, right } => {
                let left_reg = self.generate_node(&left, &mut scopes, push_block)?;
                let right_reg = self.generate_node(&right, &mut scopes, push_block)?;
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
                    _ => {
                        println!("Cannot compile binary op {:?}", op);
                        return Err(InkErr::Unimplemented);
                    }
                }
                dest
            }
            Node::FnCall { func, args } => {
                let func_reg = self.generate_node(&func, &mut scopes, push_block)?;
                let mut arg_regs = Vec::new();
                for arg in args.iter() {
                    arg_regs.push(self.generate_node(arg, &mut scopes, push_block)?);
                }
                let dest = self.iota();
                self.code.push(Inst {
                    dest,
                    op: Op::Call(func_reg, arg_regs),
                });
                dest
            }
            Node::MatchClause { target: _, expr: _ } => {
                panic!("Unexpected node in compiler: Node::MatchClause")
            }
            Node::MatchExpr { cond, clauses } => {
                let cond_reg = self.generate_node(cond, &mut scopes, push_block)?;
                let dest = self.iota();
                for (i, clause) in clauses.iter().enumerate() {
                    match clause {
                        Node::MatchClause { target, expr } => {
                            let target_reg = self.generate_node(target, &mut scopes, push_block)?;
                            // branch body is implemented as a separate Block
                            let exprlist = Node::FnLiteral {
                                args: vec![],
                                body: expr.clone(),
                            };
                            let expr_reg =
                                self.generate_node(&exprlist, &mut scopes, push_block)?;
                            self.code.push(Inst {
                                dest,
                                op: Op::CallIfEq(
                                    expr_reg,
                                    cond_reg,
                                    target_reg,
                                    clauses.len() - i - 1,
                                ),
                            });
                        }
                        _ => panic!("Unexpected node in compiler: non-MatchClause in MatchExpr"),
                    }
                }
                dest
            }
            Node::ExprList(exprs) => {
                if exprs.len() == 0 {
                    let dest = self.iota();
                    let const_dest = self.push_const(Val::Null);
                    self.code.push(Inst {
                        dest,
                        op: Op::LoadConst(const_dest),
                    });
                    dest
                } else {
                    scopes.push();
                    let mut exprlist_block =
                        Block::from_nodes(exprs.clone(), &mut scopes, push_block)?;
                    scopes.pop();

                    let mut pass_thru_names = Vec::<String>::new();
                    for (name, rec) in scopes.last() {
                        if !rec.escaped {
                            continue;
                        }

                        if rec.from_current_scope {
                            self.code.push(Inst {
                                dest: rec.reg,
                                op: Op::Escape(rec.reg),
                            });
                        } else {
                            pass_thru_names.push(name.clone());
                        }
                    }
                    for name in pass_thru_names.iter() {
                        let binds_idx = exprlist_block
                            .binds_names
                            .iter()
                            .position(|nm| nm == name)
                            .unwrap();

                        // codegen for a fake `name := name`
                        let right = Node::Ident(name.to_string());
                        let right_reg = self.generate_node(&right, &mut scopes, push_block)?;

                        // update the callee's last bind to point to the caller's correct register for
                        // the pass-thru bind variable.
                        scopes.insert(name.clone(), right_reg);
                        let last_bind = exprlist_block.binds.get_mut(binds_idx).unwrap();
                        *last_bind = right_reg;
                    }
                    let block_idx = push_block(exprlist_block);

                    let closure_dest = self.iota();
                    let const_dest = self.push_const(Val::Func(block_idx, vec![]));
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
            }
            Node::EmptyIdent => {
                let dest = self.iota();
                self.code.push(Inst { dest, op: Op::Nop });
                dest
            }
            Node::Ident(name) => match scopes.get(name) {
                Some(lookup) => {
                    if lookup.from_current_scope {
                        self.code.push(Inst {
                            dest: lookup.reg,
                            op: Op::Nop,
                        });
                        lookup.reg
                    } else {
                        let bind_idx = self.binds.len();
                        self.binds_names.push(name.clone());
                        self.binds.push(lookup.reg);

                        let dest = self.iota();
                        self.code.push(Inst {
                            dest,
                            op: Op::LoadEsc(bind_idx),
                        });
                        // There is now a local Val::Escaped pointing to the heap, so future
                        // variable accesses in this scope should not LOAD_ESC
                        scopes.insert(name.clone(), dest);
                        dest
                    }
                }
                None => {
                    println!("Could not find \"{}\" in current scope", name);
                    return Err(InkErr::UndefinedVariable);
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
            Node::ObjectEntry { key: _, val: _ } => {
                panic!("Unexpected node in compiler: Node::ObjectEntry")
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
                            let key_reg: Reg;
                            if let Node::Ident(key_name) = &**key {
                                let key_node = Node::StringLiteral(key_name.clone());
                                key_reg = self.generate_node(&key_node, &mut scopes, push_block)?;
                            } else {
                                key_reg = self.generate_node(key, &mut scopes, push_block)?;
                            }
                            let val_reg = self.generate_node(val, &mut scopes, push_block)?;
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

                    let item_reg = self.generate_node(item, &mut scopes, push_block)?;
                    let item_dest = self.iota();
                    self.code.push(Inst {
                        dest: item_dest,
                        op: Op::SetComp(dest, index_dest, item_reg),
                    });
                }
                dest
            }
            Node::FnLiteral { args, body } => {
                scopes.push();
                let mut func_block = Block::new();
                for arg in args.iter() {
                    match arg {
                        Node::Ident(name) => {
                            let arg_reg = func_block.iota();
                            scopes.insert(name.clone(), arg_reg);
                        }
                        _ => (),
                    }
                }
                match &**body {
                    Node::ExprList(exprs) => {
                        if exprs.len() == 0 {
                            // special case for _ => () which should be generated as
                            // _ => (()) (null value expression list), because we don't have an AST
                            // representation of the null () constant.
                            func_block.generate_nodes(
                                vec![Node::ExprList(vec![])],
                                &mut scopes,
                                push_block,
                            )?
                        } else {
                            func_block.generate_nodes(exprs.to_vec(), &mut scopes, push_block)?
                        }
                    }
                    _ => func_block.generate_nodes(vec![*body.clone()], &mut scopes, push_block)?,
                }
                scopes.pop();

                let mut pass_thru_names = Vec::<String>::new();
                for (name, rec) in scopes.last() {
                    if !rec.escaped {
                        continue;
                    }

                    if rec.from_current_scope {
                        self.code.push(Inst {
                            dest: rec.reg,
                            op: Op::Escape(rec.reg),
                        });
                    } else {
                        pass_thru_names.push(name.clone());
                    }
                }
                for name in pass_thru_names.iter() {
                    let binds_idx = func_block
                        .binds_names
                        .iter()
                        .position(|nm| nm == name)
                        .unwrap();

                    // codegen for a fake `name := name`
                    let right = Node::Ident(name.to_string());
                    let right_reg = self.generate_node(&right, &mut scopes, push_block)?;

                    // update the callee's last bind to point to the caller's correct register for
                    // the pass-thru bind variable.
                    scopes.insert(name.clone(), right_reg);
                    let last_bind = func_block.binds.get_mut(binds_idx).unwrap();
                    *last_bind = right_reg;
                }
                let block_idx = push_block(func_block);

                let fn_dest = self.iota();
                let const_dest = self.push_const(Val::Func(block_idx, vec![]));
                self.code.push(Inst {
                    dest: fn_dest,
                    op: Op::LoadConst(const_dest),
                });
                fn_dest
            }
        };

        return Ok(result_reg);
    }
}

pub fn generate(nodes: Vec<Node>) -> Result<Vec<Block>, InkErr> {
    let mut prog = Vec::<Block>::new();
    let mut main_scopes = ScopeStack::new();
    let mut main_block = Block::new();

    // initialize runtime preamble
    let mut builtins: HashMap<String, NativeFn> = HashMap::new();
    builtins.insert("out".to_string(), runtime::builtin_out);
    builtins.insert("char".to_string(), runtime::builtin_char);
    builtins.insert("string".to_string(), runtime::builtin_string);
    builtins.insert("len".to_string(), runtime::builtin_len);

    for (name, builtin_fn) in builtins {
        let builtin_idx = main_block.push_const(Val::NativeFunc(builtin_fn));
        let builtin_reg = main_block.iota();
        main_block.code.push(Inst {
            dest: builtin_reg,
            op: Op::LoadConst(builtin_idx),
        });
        main_scopes.insert(name, builtin_reg);
    }

    main_block.generate_nodes(nodes, &mut main_scopes, &mut |block| {
        prog.push(block);
        return prog.len();
    })?;

    // ensure main loop is first
    let mut main_prog = vec![main_block];
    main_prog.append(&mut prog);

    return Ok(main_prog);
}
