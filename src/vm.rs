use std::fmt;
use std::mem;

use crate::comp::Comp;
use crate::err::InkErr;
use crate::gen::{Block, Op, Reg, Val};
use crate::runtime;

#[derive(Debug)]
pub struct Frame {
    ip: usize, // instruction pointer
    rp: Reg,   // return register
    regs: Vec<Val>,
    binds: Vec<Val>,
    block: Block,
}

impl Frame {
    fn new(rp: Reg, block: Block) -> Frame {
        return Frame {
            ip: 0,
            rp,
            regs: vec![Val::Empty; block.slots],
            binds: vec![Val::Empty; block.binds.len()],
            block,
        };
    }
}

#[derive(Debug)]
pub struct Vm {
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

impl Val {
    fn or_from_heap<'v>(&'v self, heap: &'v Vec<Val>) -> &'v Val {
        return match self {
            Val::Escaped(heap_idx) => &heap[*heap_idx],
            _ => self,
        };
    }

    fn or_from_heap_mut<'v>(&'v mut self, heap: &'v mut Vec<Val>) -> &'v mut Val {
        return match self {
            Val::Escaped(heap_idx) => &mut heap[*heap_idx],
            _ => self,
        };
    }
}

impl Vm {
    pub fn new(prog: Vec<Block>) -> Vm {
        return Vm {
            heap: Vec::<Val>::new(),
            stack: Vec::<Frame>::new(),
            prog: prog,
        };
    }

    fn is_running(&self) -> bool {
        if self.stack.len() > 0 {
            return true;
        }
        return false;
    }

    fn should_pop_frame(&self) -> bool {
        if self.stack.len() == 0 {
            return false;
        }

        let frame = self.stack.last().unwrap();
        return frame.ip == frame.block.code.len();
    }

    pub fn run(&mut self) -> Result<(), InkErr> {
        let main_block = &self.prog.first().unwrap();
        let main_frame = Frame::new(0, (*main_block).clone());
        self.stack.push(main_frame);

        let mut maybe_callee_frame: Option<Frame>;

        while self.is_running() {
            maybe_callee_frame = None;

            let frame = self.stack.last_mut().unwrap();

            let inst = &frame.block.code[frame.ip];
            let dest = inst.dest;

            match inst.op.clone() {
                Op::Nop => (),
                Op::Mov(reg) => frame.regs[dest] = frame.regs[reg].or_from_heap(&self.heap).clone(),
                Op::Neg(reg) => {
                    frame.regs[dest] = runtime::neg(frame.regs[reg].or_from_heap(&self.heap))?
                }
                Op::Add(a, b) => {
                    frame.regs[dest] = runtime::add(
                        frame.regs[a].or_from_heap(&self.heap),
                        frame.regs[b].or_from_heap(&self.heap),
                    )?
                }
                Op::Sub(a, b) => {
                    frame.regs[dest] = runtime::sub(
                        frame.regs[a].or_from_heap(&self.heap),
                        frame.regs[b].or_from_heap(&self.heap),
                    )?
                }
                Op::Mul(a, b) => {
                    frame.regs[dest] = runtime::mul(
                        frame.regs[a].or_from_heap(&self.heap),
                        frame.regs[b].or_from_heap(&self.heap),
                    )?
                }
                Op::Div(a, b) => {
                    frame.regs[dest] = runtime::div(
                        frame.regs[a].or_from_heap(&self.heap),
                        frame.regs[b].or_from_heap(&self.heap),
                    )?
                }
                Op::Mod(a, b) => {
                    frame.regs[dest] = runtime::modulus(
                        frame.regs[a].or_from_heap(&self.heap),
                        frame.regs[b].or_from_heap(&self.heap),
                    )?
                }
                Op::And(a, b) => {
                    frame.regs[dest] = runtime::bin_and(
                        &frame.regs[a].or_from_heap(&self.heap),
                        &frame.regs[b].or_from_heap(&self.heap),
                    )?
                }
                Op::Or(a, b) => {
                    frame.regs[dest] = runtime::bin_or(
                        &frame.regs[a].or_from_heap(&self.heap),
                        &frame.regs[b].or_from_heap(&self.heap),
                    )?
                }
                Op::Xor(a, b) => {
                    frame.regs[dest] = runtime::bin_xor(
                        &frame.regs[a].or_from_heap(&self.heap),
                        &frame.regs[b].or_from_heap(&self.heap),
                    )?
                }
                Op::Gtr(a, b) => {
                    frame.regs[dest] = runtime::gtr(
                        &frame.regs[a].or_from_heap(&self.heap),
                        &frame.regs[b].or_from_heap(&self.heap),
                    )?
                }
                Op::Lss(a, b) => {
                    frame.regs[dest] = runtime::lss(
                        &frame.regs[a].or_from_heap(&self.heap),
                        &frame.regs[b].or_from_heap(&self.heap),
                    )?
                }
                Op::Eql(a, b) => {
                    frame.regs[dest] = runtime::eql(
                        &frame.regs[a].or_from_heap(&self.heap),
                        &frame.regs[b].or_from_heap(&self.heap),
                    )?
                }
                Op::Escape(reg) => {
                    let ref_idx = self.heap.len();
                    let escaping_val = &frame.regs[reg];
                    match escaping_val {
                        Val::Escaped(_) => (),
                        _ => {
                            let escaped_val =
                                mem::replace(&mut frame.regs[reg], Val::Escaped(ref_idx));
                            self.heap.push(escaped_val);
                        }
                    }
                }
                Op::Call(f_reg, arg_regs) => {
                    // TODO: tail call optimization should be implemented in the VM,
                    // not the compiler. If Op::Call is the last instruction of a Block,
                    // reuse the current stack frame position.

                    let callee_fn = frame.regs[f_reg].or_from_heap(&self.heap);
                    match callee_fn {
                        Val::Func(callee_block_idx, heap_vals) => {
                            let callee_block = &self.prog[*callee_block_idx];
                            let mut callee_frame = Frame::new(dest, callee_block.clone());

                            for (i, arg_reg) in arg_regs.iter().enumerate() {
                                callee_frame.regs[i] =
                                    frame.regs[*arg_reg].or_from_heap(&self.heap).clone();
                            }

                            for (i, val) in heap_vals.iter().enumerate() {
                                callee_frame.binds[i] = val.clone();
                            }

                            // queue up next stack frame
                            maybe_callee_frame = Some(callee_frame);
                        }
                        Val::NativeFunc(func) => {
                            let mut args = vec![Val::Empty; arg_regs.len()];
                            for (i, arg_reg) in arg_regs.iter().enumerate() {
                                args[i] = frame.regs[*arg_reg].or_from_heap(&self.heap).clone();
                            }
                            frame.regs[dest] = func(args)?;
                        }
                        _ => {
                            println!("Invalid fn: {:?}", callee_fn);
                            return Err(InkErr::InvalidFunctionCall);
                        }
                    }
                }
                Op::LoadEsc(idx) => frame.regs[dest] = frame.binds[idx].clone(),
                Op::LoadConst(idx) => {
                    let const_val = frame.block.consts[idx].clone();

                    match const_val {
                        Val::Func(callee_block_idx, heap_val_tmpl) => {
                            let callee_block = &self.prog[callee_block_idx];
                            if callee_block.binds.len() > 0 {
                                let mut heap_vals = heap_val_tmpl.clone();
                                for parent_reg_idx in callee_block.binds.iter() {
                                    heap_vals.push(frame.regs[*parent_reg_idx].clone());
                                }
                                frame.regs[dest] = Val::Func(callee_block_idx, heap_vals);
                            } else {
                                frame.regs[dest] = Val::Func(callee_block_idx, vec![]);
                            }
                        }
                        _ => frame.regs[dest] = const_val,
                    }
                }
                Op::CallIfEq(f_reg, a_reg, b_reg, skip) => {
                    let cmp_a = &frame.regs[a_reg].or_from_heap(&self.heap);
                    let cmp_b = &frame.regs[b_reg].or_from_heap(&self.heap);
                    if cmp_a.eq(&cmp_b) {
                        let callee_fn = frame.regs[f_reg].or_from_heap(&self.heap);
                        match callee_fn {
                            Val::Func(callee_block_idx, heap_vals) => {
                                let callee_block = &self.prog[*callee_block_idx];
                                let mut callee_frame = Frame::new(dest, callee_block.clone());

                                for (i, val) in heap_vals.iter().enumerate() {
                                    callee_frame.binds[i] = val.clone();
                                }

                                // queue up next stack frame
                                maybe_callee_frame = Some(callee_frame);
                            }
                            _ => {
                                println!(
                                    "CALL_IF_EQ jump point is not a function: {:?}",
                                    callee_fn
                                );
                                return Err(InkErr::InvalidFunctionCall);
                            }
                        }

                        // `skip` tells the VM to skip the next N branches
                        for _ in 0..skip {
                            'find_branch: loop {
                                frame.ip += 1;
                                if let Op::CallIfEq(_, _, _, _) = frame.block.code[frame.ip].op {
                                    break 'find_branch;
                                }
                            }
                        }
                    }
                }
                Op::MakeComp => frame.regs[dest] = Val::Comp(Comp::new()),
                Op::SetComp(comp_reg, key_reg, val_reg) => {
                    // TODO: implement for byte buffers (Val::Str's)
                    let key = frame.regs[key_reg].or_from_heap(&self.heap).clone();
                    let val = frame.regs[val_reg].or_from_heap(&self.heap).clone();

                    let comp_val = frame.regs[comp_reg].or_from_heap_mut(&mut self.heap);
                    if let Val::Comp(comp) = comp_val {
                        comp.set(&key, val);
                    } else {
                        return Err(InkErr::ExpectedCompositeValue);
                    }
                }
                Op::GetComp(comp_reg, key_reg) => {
                    let comp = frame.regs[comp_reg].or_from_heap(&self.heap);
                    let key = frame.regs[key_reg].or_from_heap(&self.heap);

                    match comp {
                        Val::Comp(comp_map) => frame.regs[dest] = comp_map.get(key),
                        _ => return Err(InkErr::ExpectedCompositeValue),
                    }
                }
            }

            frame.ip += 1;

            match maybe_callee_frame {
                Some(callee_frame) => {
                    self.stack.push(callee_frame);
                }
                None => {
                    while self.should_pop_frame() {
                        // prepare return
                        let top_frame = self.stack.last().unwrap();

                        let rp = top_frame.rp.clone();
                        let ret_reg = top_frame.block.code.last().unwrap().dest;
                        let ret_val = top_frame.regs[ret_reg].clone();
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
