use std::fs;
use std::path::PathBuf;

use rustyline::error::ReadlineError;
use rustyline::Editor;

mod analyze;
mod args;
mod comp;
mod err;
mod gen;
mod lex;
mod optimize;
mod parse;
mod runtime;
mod val;
mod vm;

const INK_VERSION: &str = "0.1.7";

fn main() {
    let opts = args::get_cli_opts();

    match opts.clone().action {
        args::Action::Eval(mode) => run_eval(mode, opts),
        args::Action::Version => print_version(),
        args::Action::Help => print_help(),
    }
}

fn print_version() {
    println!("ink v{}", INK_VERSION);
}

fn print_help() {
    println!("help...");
}

fn run_eval(mode: args::EvalMode, opts: args::Opts) {
    let result = match mode {
        args::EvalMode::RunFile(path) => eval_file(path, &opts),
        args::EvalMode::Eval(prog) => eval_string(prog, &opts),
        args::EvalMode::Repl => eval_repl(&opts),
    };

    match result {
        Err(e) => eprintln!("{:?}", e),
        _ => (),
    }
}

fn eval_file(path: PathBuf, opts: &args::Opts) -> Result<val::Val, err::InkErr> {
    let file = match fs::read_to_string(path) {
        Ok(prog) => prog,
        Err(e) => {
            eprintln!("error: {:?}", e);
            std::process::exit(1);
        }
    };

    return eval_string(file, opts);
}

fn eval_repl(opts: &args::Opts) -> Result<val::Val, err::InkErr> {
    let mut rl = Editor::<()>::new();

    let repl_do = |prog: String| -> Result<val::Val, err::InkErr> {
        let optimized_blocks = compile(prog, opts)?;
        return eval_blocks(optimized_blocks);
    };

    loop {
        match rl.readline("ink/ ") {
            Ok(line) => {
                let read_str = line.as_str();

                rl.add_history_entry(read_str);
                match repl_do(read_str.to_owned()) {
                    Ok(ret_val) => {
                        println!("{}", ret_val);
                    }
                    Err(e) => {
                        eprintln!("{:?}", e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("interrupted.");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("eof.");
                break;
            }
            Err(err) => {
                println!("readline error: {:?}", err);
                break;
            }
        }
    }

    return Ok(val::Val::Null);
}

fn compile(prog: String, opts: &args::Opts) -> Result<Vec<gen::Block>, err::InkErr> {
    let tokens = lex::tokenize(&prog)?;
    if opts.debug_lex {
        println!(":: Tokens ::");
        for (i, tok) in tokens.iter().enumerate() {
            println!("{}  {}", i, tok);
        }
    }

    let mut nodes = parse::parse(tokens)?;
    if opts.debug_parse {
        println!(":: AST nodes ::");
        for node in nodes.iter() {
            println!("{:?}", node);
        }
    }

    analyze::analyze(&mut nodes)?;
    if opts.debug_analyze {
        println!(":: Analyzed AST nodes ::");
        for node in nodes.iter() {
            println!("{:?}", node);
        }
    }

    let blocks = gen::generate(nodes)?;
    if opts.debug_compile {
        println!(":: Bytecode blocks ::");
        for (i, block) in blocks.iter().enumerate() {
            println!("#{}\n{}", i, block);
        }
    }

    let optimized_blocks = optimize::optimize(blocks);
    if opts.debug_optimize {
        println!(":: Optimized bytecode blocks ::");
        for (i, block) in optimized_blocks.iter().enumerate() {
            println!("#{}\n{}", i, block);
        }
    }

    return Ok(optimized_blocks);
}

fn eval_string(prog: String, opts: &args::Opts) -> Result<val::Val, err::InkErr> {
    let optimized_blocks = compile(prog, opts)?;
    return eval_blocks(optimized_blocks);
}

fn eval_blocks(blocks: Vec<gen::Block>) -> Result<val::Val, err::InkErr> {
    let mut machine = vm::Vm::new(blocks);
    return machine.run();
}
