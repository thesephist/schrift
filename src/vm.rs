use crate::err::InkErr;
use crate::gen::{Block, Val};
use std::fmt;

#[derive(Debug)]
pub struct Frame {
    rbp: usize, // return address (block pointer)
    regs: Vec<Val>,
    block: Block,
}

impl Frame {
    fn new(rbp: usize, block: Block) -> Frame {
        return Frame {
            rbp: rbp,
            regs: vec![Val::Empty; block.slots],
            block: block,
        };
    }
}

#[derive(Debug)]
pub struct Vm {
    bp: usize,      // block counter
    heap: Vec<Val>, // escaped (bind) values
    stack: Vec<Frame>,
    prog: Vec<Block>,
}

impl fmt::Display for Vm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "bp: {}", self.bp)?;
        writeln!(f, "heap:")?;
        for val in &self.heap {
            writeln!(f, "  {:?}", val)?;
        }
        writeln!(f, "stack:")?;
        for frame in &self.stack {
            writeln!(f, "  {:?}", frame)?;
        }
        writeln!(f, "prog:")?;
        for block in &self.prog {
            writeln!(f, "  {:?}", block)?;
        }
        writeln!(f, "")
    }
}

impl Vm {
    pub fn new(prog: Vec<Block>) -> Vm {
        return Vm {
            bp: 0,
            heap: Vec::<Val>::new(),
            stack: Vec::<Frame>::new(),
            prog: prog,
        };
    }

    pub fn run(&mut self) -> Result<(), InkErr> {
        let main_block = &self.prog[self.bp];
        let main_frame = Frame::new(0, main_block.clone());
        self.stack.push(main_frame);

        while self.stack.len() > 0 {
            let frame = self.stack.last().unwrap();
            for inst in &frame.block.code {
                match inst {
                    _ => println!("Unknown instruction {:?}", inst),
                }
            }
            self.stack.pop();
        }

        return Ok(());
    }
}
