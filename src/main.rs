use std::fs;
use std::path::PathBuf;

mod analyze;
mod args;
mod err;
mod gen;
mod lex;
mod parse;
mod vm;

const INK_VERSION: &str = "0.1.7";

fn main() {
    let opts = args::get_cli_opts();

    match opts.action {
        args::Action::Eval(mode) => run_eval(mode),
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

fn run_eval(mode: args::EvalMode) {
    let result = match mode {
        args::EvalMode::RunFile(path) => eval_file(path),
        args::EvalMode::Eval(prog) => eval_string(prog),
        args::EvalMode::Repl => Ok(println!("repl")),
    };

    match result {
        Err(e) => eprintln!("{:?}", e),
        _ => (),
    }
}

fn eval_file(path: PathBuf) -> Result<(), err::InkErr> {
    let file = match fs::read_to_string(path) {
        Ok(prog) => prog,
        Err(e) => {
            eprintln!("error: {:?}", e);
            std::process::exit(1);
        }
    };

    return eval_string(file);
}

fn eval_string(prog: String) -> Result<(), err::InkErr> {
    let tokens = lex::tokenize(&prog)?;
    // println!(":: Tokens ::");
    // for (i, tok) in tokens.iter().enumerate() {
    //     println!("{}  {}", i, tok);
    // }

    let mut nodes = parse::parse(tokens)?;
    // println!(":: AST nodes ::");
    // for node in nodes.iter() {
    //     println!("{:?}", node);
    // }

    analyze::analyze(&mut nodes)?;
    // println!(":: Analyzed AST nodes ::");
    // for node in nodes.iter() {
    //     println!("{:?}", node);
    // }

    let insts = gen::generate(nodes)?;
    // println!(":: Instructions ::");
    // for inst in insts.iter() {
    //     println!("{:?}", inst);
    // }

    let mut machine = vm::Vm::new(insts);
    return machine.run();
}
