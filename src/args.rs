use std::env;
use std::path::PathBuf;

// Ink CLI has 3 modes of operation.
// 1. "Run" which runs a file with arguments
// 2. "Eval" which evals from a CLI argument
// 3. "Stdin" which evals from stdin
// 4. "Repl" which opens a read-eval-print loop
#[derive(Debug, Clone)]
pub enum EvalMode {
    RunFile(PathBuf),
    Eval(String),
    // Stdin,
    Repl,
}

#[derive(Clone)]
pub enum Action {
    Eval(EvalMode),
    Version,
    Help,
}

#[derive(Clone)]
pub struct Opts {
    pub action: Action,

    pub debug_lex: bool,
    pub debug_parse: bool,
    pub debug_analyze: bool,
    pub debug_compile: bool,
}

pub fn get_cli_opts() -> Opts {
    let all_args: Vec<String> = env::args().collect();
    let args = &all_args[1..];

    let mut opts = Opts {
        action: Action::Help,

        debug_lex: false,
        debug_parse: false,
        debug_analyze: false,
        debug_compile: false,
    };

    opts.action = if args.len() == 0 {
        Action::Eval(EvalMode::Repl)
    } else {
        match &(args[0][..]) {
            "version" => Action::Version,
            "help" => Action::Help,
            "eval" => {
                if args.len() >= 2 {
                    let prog = String::from(args[1].clone());
                    Action::Eval(EvalMode::Eval(prog))
                } else {
                    Action::Help
                }
            }
            path_str => {
                let mut path = PathBuf::new();
                path.push(path_str);
                Action::Eval(EvalMode::RunFile(path))
            }
        }
    };

    for arg in args.iter() {
        if arg.starts_with("--") {
            let flag_str = &arg[2..];
            match flag_str {
                "debug-lex" => opts.debug_lex = true,
                "debug-parse" => opts.debug_parse = true,
                "debug-analyze" => opts.debug_analyze = true,
                "debug-compile" => opts.debug_compile = true,
                _ => (),
            }
        }

        if arg.starts_with("-") {
            let flag_str = &arg[1..];
            match flag_str {
                "Dl" => opts.debug_lex = true,
                "Dp" => opts.debug_parse = true,
                "Da" => opts.debug_analyze = true,
                "Dc" => opts.debug_compile = true,
                _ => (),
            }
        }
    }

    return opts;
}
