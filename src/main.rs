// (setq rustic-run-arguments "-- c:/tmp/simple.lox")
use std::{env, fs, process::exit};

use vm::{InterpretResult, VM};

mod chunk;
mod compiler;
mod debug;
mod op_code;
mod scanner;
mod vm;

fn main() {
    let arguments: Vec<String> = env::args().collect();
    if arguments.len() >= 2 {
        run_file(&arguments[arguments.len() - 1]);
    } else {
        eprintln!("missing filename");
    }
}

fn run_file(filename: &str) {
    let file = fs::read_to_string(filename).expect("file not found");
    let mut vm = VM::new();
    match vm.interpret(file) {
        InterpretResult::Ok => (),
        InterpretResult::CompileError => exit(65),
        InterpretResult::RuntimeError => exit(70),
    }
}
