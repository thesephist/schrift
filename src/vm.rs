use std::fmt;

use crate::err::InkErr;
use crate::gen::{Block, Op, Reg, Val};
use crate::runtime;

#[derive(Debug)]
pub struct Frame {
    ip: usize, // instruction pointer
    rp: Reg,   // return register
    regs: Vec<Val>,
    block: Block,
}

impl Frame {
    fn new(rp: Reg, block: Block) -> Frame {
        return Frame {
            ip: 0,
            rp,
            regs: vec![Val::Empty; block.slots],
            block,
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

    fn is_running(&self) -> bool {
        if self.stack.len() == 0 {
            return false;
        }

        let top_frame = self.stack.last().unwrap();
        return top_frame.ip < top_frame.block.code.len();
    }

    pub fn run(&mut self) -> Result<(), InkErr> {
        let main_block = &self.prog.first().unwrap();
        let main_frame = Frame::new(0, (*main_block).clone());
        self.stack.push(main_frame);

        let mut maybe_callee_frame: Option<Frame>;

        // Core while-switch instruction dispatch loop
        while self.is_running() {
            maybe_callee_frame = None;

            let frame = self.stack.last_mut().unwrap();

            let inst = &frame.block.code[frame.ip];
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
                    match callee_fn {
                        Val::Func(callee_block_idx) => {
                            let callee_block = &self.prog[callee_block_idx.clone()];
                            let mut callee_frame = Frame::new(dest, callee_block.clone());

                            for (i, arg_reg) in arg_regs.iter().enumerate() {
                                callee_frame.regs[i] = frame.regs[arg_reg.clone()].clone();
                            }

                            // queue up next stack frame
                            maybe_callee_frame = Some(callee_frame);
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

            frame.ip += 1;

            match maybe_callee_frame {
                Some(callee_frame) => {
                    self.stack.push(callee_frame);
                }
                None => {
                    if frame.ip == frame.block.code.len() {
                        // prepare return
                        let rp = frame.rp.clone();
                        let ret_reg = frame.block.code.last().unwrap().dest;
                        let ret_val = frame.regs[ret_reg].clone();

                        self.stack.pop();
                        match self.stack.last_mut() {
                            Some(frame) => frame.regs[rp] = ret_val,
                            None => (),
                        }
                    }
                }
            }
        }

        return Ok(());
    }
}
