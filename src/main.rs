mod args;
mod lex;

const INK_VERSION: &str = "0.1.7";

fn main() {
    let opts = args::get_cli_opts();

    match opts.action {
        args::Action::Eval(mode) => println!("eval mode {:?}", mode),
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
