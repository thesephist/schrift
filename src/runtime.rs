use std::io::{self, Write};

use std::cell::RefCell;
use std::rc::Rc;

use crate::err::InkErr;
use crate::comp::Comp;
use crate::val::Val;

pub fn neg(v: &Val) -> Result<Val, InkErr> {
    let result = match v {
        Val::Number(n) => Val::Number(-n),
        Val::Bool(b) => Val::Bool(!b),
        _ => return Err(InkErr::InvalidOperand),
    };

    return Ok(result);
}

pub fn add(a: &Val, b: &Val) -> Result<Val, InkErr> {
    let result = match a {
        Val::Number(num_a) => match b {
            Val::Number(num_b) => Val::Number(num_a + num_b),
            _ => return Err(InkErr::InvalidOperand),
        },
        Val::Str(bytes_a) => match b {
            Val::Str(bytes_b) => {
                let mut append_to = bytes_a.clone();
                append_to.extend(bytes_b.clone());
                return Ok(Val::Str(append_to));
            }
            _ => return Err(InkErr::InvalidOperand),
        },
        Val::Bool(bool_a) => match b {
            Val::Bool(bool_b) => return Ok(Val::Bool(*bool_a || *bool_b)),
            _ => return Err(InkErr::InvalidOperand),
        },
        _ => return Err(InkErr::InvalidOperand),
    };

    return Ok(result);
}

pub fn sub(a: &Val, b: &Val) -> Result<Val, InkErr> {
    let result = match a {
        Val::Number(num_a) => match b {
            Val::Number(num_b) => Val::Number(num_a - num_b),
            _ => return Err(InkErr::InvalidOperand),
        },
        _ => return Err(InkErr::InvalidOperand),
    };

    return Ok(result);
}

pub fn mul(a: &Val, b: &Val) -> Result<Val, InkErr> {
    let result = match a {
        Val::Number(num_a) => match b {
            Val::Number(num_b) => Val::Number(num_a * num_b),
            _ => return Err(InkErr::InvalidOperand),
        },
        Val::Bool(bool_a) => match b {
            Val::Bool(bool_b) => return Ok(Val::Bool(*bool_a && *bool_b)),
            _ => return Err(InkErr::InvalidOperand),
        },
        _ => return Err(InkErr::InvalidOperand),
    };

    return Ok(result);
}

pub fn div(a: &Val, b: &Val) -> Result<Val, InkErr> {
    let result = match a {
        Val::Number(num_a) => match b {
            Val::Number(num_b) => Val::Number(num_a / num_b),
            _ => return Err(InkErr::InvalidOperand),
        },
        _ => return Err(InkErr::InvalidOperand),
    };

    return Ok(result);
}

pub fn modulus(a: &Val, b: &Val) -> Result<Val, InkErr> {
    let result = match a {
        Val::Number(num_a) => match b {
            Val::Number(num_b) => Val::Number(num_a % num_b),
            _ => return Err(InkErr::InvalidOperand),
        },
        _ => return Err(InkErr::InvalidOperand),
    };

    return Ok(result);
}

// zero-extend a given Vec<u8> (backing vector of a Val::Str)
// to be used in binary bitwise operations on byte strings
fn zero_extend(v: &Vec<u8>, max: usize) -> Vec<u8> {
    let mut extended: Vec<u8> = vec![0; max];
    for (i, b) in v.iter().enumerate() {
        extended[i] = *b;
    }
    return extended;
}

// return the max length of two vectors. Utility function used in
// bitwise binary operations on byte strings
fn max_len(a: &Vec<u8>, b: &Vec<u8>) -> usize {
    let asize = a.len();
    let bsize = b.len();
    return if asize < bsize { bsize } else { asize };
}

pub fn bin_and(a: &Val, b: &Val) -> Result<Val, InkErr> {
    let result = match a {
        Val::Number(num_a) => match b {
            Val::Number(num_b) => Val::Number((*num_a as i64 & *num_b as i64) as f64),
            _ => return Err(InkErr::InvalidOperand),
        },
        Val::Str(str_a) => match b {
            Val::Str(str_b) => {
                let max = max_len(str_a, str_b);

                let a = zero_extend(str_a, max);
                let b = zero_extend(str_b, max);
                let mut c: Vec<u8> = vec![0; max];

                for i in 0..c.len() {
                    c[i] = a[i] & b[i];
                }
                Val::Str(c)
            }
            _ => return Err(InkErr::InvalidOperand),
        },
        Val::Bool(bool_a) => match b {
            Val::Bool(bool_b) => return Ok(Val::Bool(*bool_a && *bool_b)),
            _ => return Err(InkErr::InvalidOperand),
        },
        _ => return Err(InkErr::InvalidOperand),
    };

    return Ok(result);
}

pub fn bin_or(a: &Val, b: &Val) -> Result<Val, InkErr> {
    let result = match a {
        Val::Number(num_a) => match b {
            Val::Number(num_b) => Val::Number((*num_a as i64 | *num_b as i64) as f64),
            _ => return Err(InkErr::InvalidOperand),
        },
        Val::Str(str_a) => match b {
            Val::Str(str_b) => {
                let max = max_len(str_a, str_b);

                let a = zero_extend(str_a, max);
                let b = zero_extend(str_b, max);
                let mut c: Vec<u8> = vec![0; max];

                for i in 0..c.len() {
                    c[i] = a[i] | b[i];
                }
                Val::Str(c)
            }
            _ => return Err(InkErr::InvalidOperand),
        },
        Val::Bool(bool_a) => match b {
            Val::Bool(bool_b) => return Ok(Val::Bool(*bool_a || *bool_b)),
            _ => return Err(InkErr::InvalidOperand),
        },
        _ => return Err(InkErr::InvalidOperand),
    };

    return Ok(result);
}

pub fn bin_xor(a: &Val, b: &Val) -> Result<Val, InkErr> {
    let result = match a {
        Val::Number(num_a) => match b {
            Val::Number(num_b) => Val::Number((*num_a as i64 ^ *num_b as i64) as f64),
            _ => return Err(InkErr::InvalidOperand),
        },
        Val::Str(str_a) => match b {
            Val::Str(str_b) => {
                let max = max_len(str_a, str_b);

                let a = zero_extend(str_a, max);
                let b = zero_extend(str_b, max);
                let mut c: Vec<u8> = vec![0; max];

                for i in 0..c.len() {
                    c[i] = a[i] ^ b[i];
                }
                Val::Str(c)
            }
            _ => return Err(InkErr::InvalidOperand),
        },
        Val::Bool(bool_a) => match b {
            Val::Bool(bool_b) => return Ok(Val::Bool(*bool_a != *bool_b)),
            _ => return Err(InkErr::InvalidOperand),
        },
        _ => return Err(InkErr::InvalidOperand),
    };

    return Ok(result);
}

pub fn gtr(a: &Val, b: &Val) -> Result<Val, InkErr> {
    let result = match a {
        Val::Number(num_a) => match b {
            Val::Number(num_b) => Val::Bool(num_a > num_b),
            _ => return Err(InkErr::InvalidOperand),
        },
        Val::Str(_bytes_a) => match b {
            Val::Str(_bytes_b) => {
                return Err(InkErr::InvalidOperand);
            }
            _ => return Err(InkErr::InvalidOperand),
        },
        _ => return Err(InkErr::InvalidOperand),
    };

    return Ok(result);
}

pub fn lss(a: &Val, b: &Val) -> Result<Val, InkErr> {
    let result = match a {
        Val::Number(num_a) => match b {
            Val::Number(num_b) => Val::Bool(num_a < num_b),
            _ => return Err(InkErr::InvalidOperand),
        },
        Val::Str(_bytes_a) => match b {
            Val::Str(_bytes_b) => {
                return Err(InkErr::InvalidOperand);
            }
            _ => return Err(InkErr::InvalidOperand),
        },
        _ => return Err(InkErr::InvalidOperand),
    };

    return Ok(result);
}

pub fn eql(a: &Val, b: &Val) -> Result<Val, InkErr> {
    return Ok(Val::Bool(a.eq(b)));
}

// runtime builtins

pub fn builtin_out(args: Vec<Val>) -> Result<Val, InkErr> {
    if args.len() < 1 {
        return Err(InkErr::NotEnoughArguments);
    }

    let out_arg = &args[0];
    return match out_arg {
        Val::Str(s) => match io::stdout().write_all(s) {
            Err(_) => return Err(InkErr::IOError),
            _ => return Ok(out_arg.clone()),
        },
        _ => Err(InkErr::InvalidArguments),
    };
}

pub fn builtin_char(args: Vec<Val>) -> Result<Val, InkErr> {
    if args.len() < 1 {
        return Err(InkErr::NotEnoughArguments);
    }

    let out_arg = &args[0];
    return match out_arg {
        Val::Number(n) => {
            let str_result = ((n.clone() as u8) as char).to_string().as_bytes().to_vec();
            return Ok(Val::Str(str_result));
        }
        _ => Err(InkErr::InvalidArguments),
    };
}

pub fn builtin_string(args: Vec<Val>) -> Result<Val, InkErr> {
    if args.len() < 1 {
        return Err(InkErr::NotEnoughArguments);
    }

    let arg = &args[0];
    let ink_str_bytes = match &arg {
        Val::Str(s) => s.clone(),
        _ => arg.to_ink_string().as_bytes().to_vec(),
    };

    return Ok(Val::Str(ink_str_bytes));
}

pub fn builtin_len(args: Vec<Val>) -> Result<Val, InkErr> {
    if args.len() < 1 {
        return Err(InkErr::NotEnoughArguments);
    }

    let arg = &args[0];
    let length = match &arg {
        Val::Str(s) => s.len(),
        Val::Comp(comp_rc) => comp_rc.borrow().len(),
        _ => return Err(InkErr::InvalidArguments),
    };

    return Ok(Val::Number(length as f64));
}

pub fn builtin_load(args: Vec<Val>) -> Result<Val, InkErr> {
    if args.len() < 1 {
        return Err(InkErr::NotEnoughArguments);
    }

    let arg = &args[0];
    return match &arg {
        Val::Str(_path_str) => {
            println!("loading {}", arg);
            /*
             * TODO: Ink load() builtin implementation design:
             *
             * 0. For deduplication of imports / recursive imports, create and keep a Map<Path,
             *    Comp> per-VM. A VM represents a single execution thread, so all Context*
             *    variables live there.
             * 1. Against the same main `Block`, but different root `ScopeStack`, `generate_node`
             *    the program from the file as an `ExprList`. This should result in two things: (a)
             *    the bytecode from this new module gets compiled into the same `Vec<Block>` for
             *    the VM to execute, and (b) we end up with a top-level lexical `Scope` that maps
             *    global names (importable names) to registers where the live in the `ExprList`'s'
             *    execution stack.
             * 2. Eval the compiled `ExprList` blocks. This allocates into the globally named
             *    registers in the global scope the right values.
             * 3. Create a `Comp` where keys of the global `ScopeStack` scope map to values in the
             *    corresponding registers. This is the map to be imported. Return this `Comp` from
             *    load. Optionally (0), update the per-VM import map for future imports of the same
             *    program file.
             *
             * In this design, a single VM contains all bytecode for all imported modules, but
             * because jumps (Call instructions) can't cross these module boundaries if compiled
             * correctly, this works efficiently.
             */
            Ok(Val::Comp(Rc::new(RefCell::new(Comp::new()))))
        },
        _ => Err(InkErr::InvalidArguments),
    };
}
