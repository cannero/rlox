// (setq rustic-run-arguments "-- c:/tmp/simple.lox")
use std::{env, fs::{self, File}, io::Write, process::exit};

use compiler::compile;
use debug::Debugger;
use value::Function;
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
        let filename = &arguments[arguments.len() - 1];
    
        let debug_switch = arguments.len() >= 3
            && (arguments.contains(&"--debug".to_string())
                || arguments.contains(&"-d".to_string()));

        if arguments.len() >= 3 && arguments.contains(&"--run".to_string()) {
            let file = fs::read(filename).expect(&format!("file '{}' not found", filename));
            run(file, debug_switch);
        } else {
            let file = fs::read_to_string(filename).expect(&format!("file '{}' not found", filename));
            if arguments.contains(&"--compile".to_string()) {
                only_compile(filename, file, debug_switch);
            } else {
                compile_and_run(file, debug_switch);
            }
        }
    } else {
        eprintln!("missing filename");
    }
}

fn compile_and_run(file: String, debug: bool) {
    let mut vm = VM::new();
    match vm.interpret(file, debug) {
        InterpretResult::Ok => (),
        InterpretResult::CompileError => exit(65),
        InterpretResult::RuntimeError => exit(70),
    }
}

fn only_compile(filepath: &str, file: String, debug: bool) {
    match compile(file, debug) {
        Ok(function) => {
            if debug {
                let mut debugger = Debugger::new();
                debugger.disassemble_chunk(&function, "code");
            }

            let path = filepath.replace(".lox", ".loxer");
            let data = bson::to_vec(&function).expect("Serialize to bson failed.");
            let mut file = File::create(&path).expect("loxer file creation failed.");
            file.write_all(&data).expect("loxer file could not be written.");
            println!("file {} written", path);
        }
        Err(_) => eprintln!("compilation failed"),
    }
}

fn run(file: Vec<u8>, debug: bool) {
    let function : Function = bson::from_slice(&file).unwrap();
    let mut vm = VM::new();
    match vm.run_function(function, debug) {
        InterpretResult::Ok => (),
        InterpretResult::CompileError => exit(65),
        InterpretResult::RuntimeError => exit(70),
    }
}
