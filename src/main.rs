// (setq rustic-run-arguments "-- c:/tmp/simple.lox")
use std::{env, fs, process::exit};

use vm::{InterpretResult, VM};

mod chunk;
mod compiler;
mod debug;
mod op_code;
mod scanner;
mod value;
mod vm;

fn main() {
    let arguments: Vec<String> = env::args().collect();
    if arguments.len() >= 2 {
        let debug_switch = arguments.len() >= 3 && (arguments.contains(&"--debug".to_string()) || arguments.contains(&"-d".to_string()));
        run_file(&arguments[arguments.len() - 1], debug_switch);
    } else {
        eprintln!("missing filename");
    }
}

fn run_file(filename: &str, debug: bool) {
    let file = fs::read_to_string(filename).expect("file not found");
    let mut vm = VM::new();
    match vm.interpret(file, debug) {
        InterpretResult::Ok => (),
        InterpretResult::CompileError => exit(65),
        InterpretResult::RuntimeError => exit(70),
    }
}
