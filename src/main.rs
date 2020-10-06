use std::fs;
use std::path::PathBuf;

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
        args::EvalMode::RunFile(path) => eval_file(path, opts),
        args::EvalMode::Eval(prog) => eval_string(prog, opts),
        args::EvalMode::Repl => Ok(println!("repl")),
    };

    match result {
        Err(e) => eprintln!("{:?}", e),
        _ => (),
    }
}

fn eval_file(path: PathBuf, opts: args::Opts) -> Result<(), err::InkErr> {
    let file = match fs::read_to_string(path) {
        Ok(prog) => prog,
        Err(e) => {
            eprintln!("error: {:?}", e);
            std::process::exit(1);
        }
    };

    return eval_string(file, opts);
}

fn eval_string(prog: String, opts: args::Opts) -> Result<(), err::InkErr> {
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

    let mut machine = vm::Vm::new(optimized_blocks);
    return machine.run();
}
