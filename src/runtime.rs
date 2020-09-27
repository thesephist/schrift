use std::io::{self, Write};

use crate::err::InkErr;
use crate::gen::Val;

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

pub fn bin_and(a: &Val, b: &Val) -> Result<Val, InkErr> {
    let result = match a {
        Val::Number(num_a) => match b {
            Val::Number(num_b) => Val::Number((*num_a as i64 & *num_b as i64) as f64),
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
