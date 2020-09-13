use crate::err::InkErr;
use crate::gen::Prog;

#[derive(Debug)]
pub struct Vm {
    pc: usize,
    prog: Prog,
}

impl Vm {
    pub fn new(prog: Prog) -> Vm {
        return Vm { pc: 0, prog: prog };
    }

    pub fn run(&mut self) -> Result<(), InkErr> {
        while self.pc < self.prog.len() {
            match self.prog[self.pc] {
                _ => println!("Unrecognized instruction: {:?}", self.prog[self.pc]),
            }
            self.pc += 1;
        }

        return Ok(());
    }
}
