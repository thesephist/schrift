use std::fs;
use std::path::PathBuf;

mod args;
mod lex;

const INK_VERSION: &str = "0.1.7";

fn main() {
    let opts = args::get_cli_opts();

    match opts.action {
        args::Action::Eval(mode) => eval(mode),
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

fn eval(mode: args::EvalMode) {
    match mode {
        args::EvalMode::RunFile(path) => eval_file(path),
        args::EvalMode::Eval(prog) => eval_string(prog),
        args::EvalMode::Repl => println!("repl"),
    }
}

fn eval_file(path: PathBuf) {
    let file = match fs::read_to_string(path) {
        Ok(prog) => prog,
        Err(e) => {
            eprintln!("error: {:?}", e);
            std::process::exit(1);
        }
    };

    eval_string(file);
}

fn eval_string(prog: String) {
    let tokens = lex::tokenize(&prog);
    match tokens {
        Ok(ts) => {
            for tok in ts.iter() {
                println!("{}", tok);
            }
        }
        Err(e) => eprintln!("{:?}", e),
    }
}
