use std::fmt;

use crate::err::InkErr;
use crate::gen::{Block, Op, Val};
use crate::runtime;

#[derive(Debug)]
pub struct Frame {
    ip: usize, // instruction pointer
    regs: Vec<Val>,
    block: Block,
}

impl Frame {
    fn new(block: Block) -> Frame {
        return Frame {
            ip: 0,
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
        let main_block = &self.prog.first().unwrap();
        let main_frame = Frame::new((*main_block).clone());
        self.stack.push(main_frame);

        while self.stack.len() > 0 {
            let frame = self.stack.last_mut().unwrap();
            for inst in &frame.block.code {
                let dest = inst.dest;
                match inst.op.clone() {
                    Op::Nop => (),
                    Op::Mov(reg) => frame.regs[dest] = frame.regs[reg].clone(),
                    Op::Add(a, b) => {
                        frame.regs[dest] = runtime::add(&frame.regs[a], &frame.regs[b])?.clone()
                    }
                    Op::Call(f_reg, arg_regs) => {
                        // TODO: tail call optimization should be implemented in the VM,
                        // not the compiler. If Op::Call is the last instruction of a Block,
                        // reuse the current stack frame position.
                        let callee_fn = &frame.regs[f_reg];
                        std::thread::sleep(std::time::Duration::from_millis(10));
                        match callee_fn {
                            Val::Func(callee_block_idx) => {
                                let callee_block = &self.prog[callee_block_idx.clone()];
                                let mut callee_frame = Frame::new(callee_block.clone());

                                for (i, arg_reg) in arg_regs.iter().enumerate() {
                                    callee_frame.regs[i] = frame.regs[arg_reg.clone()].clone();
                                }

                                // TODO: push the stack and restart vm dispatch loop
                                // self.stack.push(callee_frame);
                            }
                            Val::NativeFunc(func) => {
                                let args = arg_regs
                                    .iter()
                                    .map(|reg| frame.regs[reg.clone()].clone())
                                    .collect();
                                frame.regs[dest] = func(args)?;
                            }
                            _ => return Err(InkErr::InvalidFunctionCall),
                        }
                    }
                    Op::LoadConst(idx) => frame.regs[dest] = frame.block.consts[idx].clone(),
                    _ => println!("Unknown instruction {:?}", inst),
                }
            }
            self.stack.pop();
        }

        return Ok(());
    }
}
