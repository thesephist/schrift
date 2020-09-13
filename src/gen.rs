use crate::err::InkErr;
use crate::parse::Node;

#[allow(unused)]
#[derive(Debug)]
pub enum Inst {
    Nop,

    Mov,

    Add,
    Sub,
    Mul,
    Div,

    SetVal,
    GetVal,

    Jmp,
    Equ,
    Lth,
    Gth,

    Call,
    Ret,
}

pub type Prog = Vec<Inst>;

pub fn generate(nodes: Vec<Node>) -> Result<Prog, InkErr> {
    let mut program = Vec::<Inst>::new();
    for node in nodes.iter() {
        program.extend(generate_node(node.clone())?);
    }
    return Ok(program);
}

fn generate_node(node: Node) -> Result<Prog, InkErr> {
    return Ok(vec![Inst::Nop]);
}
