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

    LoadArg(usize),
    LoadConst(usize),
    LoadBind(usize),
    LoadBlock(Reg),

    Call(Reg),
    CallIfEq(Reg, Reg, Reg),

    MakeComp,
    SetComp(Reg, Reg, Reg),
    GetComp(Reg, Reg),
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
}

impl Block {
    fn new() -> Block {
        return Block {
            slots: 0,
            consts: Vec::new(),
            binds: Vec::new(),
            ret: 0,
            code: vec![],
        };
    }

    fn generate_node(&mut self, node: &Node) -> Result<(), InkErr> {
        self.code.push(Inst {
            dest: 0,
            op: Op::Nop,
        });
        return Ok(());
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
