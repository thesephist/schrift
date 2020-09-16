use std::io::{self, Write};

use crate::err::InkErr;
use crate::gen::Val;

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
        _ => return Err(InkErr::InvalidOperand),
    };

    return Ok(result);
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

    let out_arg = &args[0];
    let ink_str_bytes = match &out_arg {
        Val::Str(s) => s.clone(),
        Val::Number(n) => n.to_string().as_bytes().to_vec(),
        Val::Empty => "_".to_string().as_bytes().to_vec(),
        Val::Bool(v) => v.to_string().as_bytes().to_vec(),
        Val::Null => "()".as_bytes().to_vec(),
        _ => return Err(InkErr::Unimplemented),
    };

    return Ok(Val::Str(ink_str_bytes));
}
