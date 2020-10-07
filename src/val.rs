use std::fmt;

use crate::comp::Comp;
use crate::err::InkErr;

pub type NativeFn = fn(Vec<Val>) -> Result<Val, InkErr>;

#[derive(Debug, Clone)]
pub enum Val {
    Empty,
    Number(f64),
    Str(Vec<u8>),
    Bool(bool),
    Null,
    Comp(Comp),
    Func(usize, Vec<Val>),
    NativeFunc(NativeFn),

    // NOTE: Slightly outdated.
    //
    // Val::Escaped(Arc<Val>) is a proxy value placed in registers to tell the VM that the register
    // value has been moved to the VM's heap.
    //
    // At compile time:
    // ===============
    //
    // When a variable in scope A register R is determined to have escaped by a closure with scope
    // B (or a composite), the compiler makes these changes:
    //
    // X. In Block A, add instruction [@R ESCAPE] which tells the VM to move the value to the VM
    //    heap
    // X. Add a reference (TBD) to Block B's Block::bind vector that will runtime-reference
    //    register @R in A.
    // X. In Block B, add instruction [@? LOAD_ESC N] when loading the closed-over variable, which
    //    will pull from the runtime-created vec of heap pointers (Vec::Escaped's).
    //
    // At runtime:
    // ===========
    //
    // When the VM LOAD_CONST's a function literal:
    //
    // X. If the Val::Func's block has any closed-over variable registers in Block::bind, /clone/
    //    the Val::Func and add to it the runtime-determined Vec::Escaped's sitting in those
    //    registers. This produces a new "function object" which is the closure closing over
    //    runtime values sitting on the VM heap.
    //
    // When the VM CALL's a Val::Func:
    //
    // 1. If the Val::Func has any heap pointers in its heap pointer (closed-over variables)
    //    vector, make those Val::Escaped's (heap pointers) available in the vm::Frame in a
    //    predictable way to the frame's bytecode.
    Escaped(usize),
}

impl fmt::Display for Val {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Val::Empty | Val::Number(_) | Val::Bool(_) | Val::Null => {
                write!(f, "{}", self.to_ink_string().replace("'", "\\'"))
            }
            Val::Str(_) => write!(f, "\'{}\'", self.to_ink_string()),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl Val {
    pub fn eq(&self, other: &Val) -> bool {
        match other {
            Val::Empty => true,
            _ => match &self {
                Val::Empty => true,
                Val::Number(a) => {
                    if let Val::Number(b) = other {
                        return a == b;
                    }
                    return false;
                }
                Val::Str(a) => {
                    if let Val::Str(b) = other {
                        return a == b;
                    }
                    return false;
                }
                Val::Bool(a) => {
                    if let Val::Bool(b) = other {
                        return a == b;
                    }
                    return false;
                }
                Val::Null => match other {
                    Val::Null => true,
                    _ => false,
                },
                // TODO: implement for Val::Comp
                _ => true,
            },
        }
    }

    pub fn to_ink_string(&self) -> String {
        match self {
            Val::Empty => "_".to_string(),
            Val::Number(n) => format!("{}", n),
            Val::Str(a) => String::from_utf8_lossy(a).into_owned(),
            Val::Bool(a) => {
                if *a {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            Val::Null => "()".to_string(),
            Val::Func(_, _) | Val::NativeFunc(_) => "(function)".to_string(),
            // TODO: to_ink_string(Val::Comp)
            _ => panic!("Tried to convert unknown Ink value {:?} to string", self),
        }
    }

    // Coerce an Ink value into a usize when possible to index into a byte string
    pub fn index_coerce(&self) -> Result<usize, InkErr> {
        return match self {
            Val::Number(n) => Ok(*n as usize),
            Val::Str(s) => {
                let as_string = match String::from_utf8(s.to_owned()) {
                    Ok(str) => str,
                    Err(_) => return Err(InkErr::ExpectedIntegerIndex),
                };
                match as_string.parse::<usize>() {
                    Ok(index) => Ok(index),
                    Err(_) => Err(InkErr::ExpectedIntegerIndex),
                }
            }
            _ => Err(InkErr::ExpectedIntegerIndex),
        };
    }
}

pub fn set_on_bytestring(s: &mut Vec<u8>, key: &Val, val: Val) -> Result<(), InkErr> {
    let index = key.index_coerce()?;
    let mut appendee = match val {
        Val::Str(s) => s,
        _ => return Err(InkErr::ExpectedString),
    };

    if index > s.len() {
        return Err(InkErr::IndexOutOfBounds);
    }
    if index == s.len() {
        s.append(&mut appendee);
    }

    // ensure backing string buffer is large enough
    if s.len() < index + appendee.len() {
        let mut filler = vec![0; index + appendee.len() - s.len()];
        s.append(&mut filler);
    }
    // mutating vec internaly
    for i in 0..appendee.len() {
        s[index + i] = appendee[i];
    }

    return Ok(());
}

pub fn get_from_bytestring(s: &Vec<u8>, key: &Val) -> Result<Val, InkErr> {
    let index = key.index_coerce()?;
    if index > s.len() {
        return Ok(Val::Null);
    }

    let char_u8 = s[index];
    return Ok(Val::Str(vec![char_u8]));
}
