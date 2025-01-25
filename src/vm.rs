use crate::{chunk::Chunk, compiler::compile, op_code::OpCode};

pub struct VM {
    ip: usize,
    stack: Vec<f32>,
}

#[derive(Debug)]
pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

macro_rules! binary_op {
    ($vm:ident, +) => {{
        let b = $vm.pop();
        let a = $vm.pop();
        $vm.push(a + b);
    }};
    ($vm:ident, $op:tt) => {{
        let b = $vm.pop();
        let a = $vm.pop();
        $vm.push(a $op b);
    }};
}

impl VM {
    
    pub fn new() -> Self {
        Self { ip: 0, stack: vec![] }
    }

    pub fn interpret(&mut self, source: String) -> InterpretResult {
        match compile(source) {
            Ok(chunk) => self.run(chunk),
            Err(_) => InterpretResult::CompileError,
        }
    }

    fn run(&mut self, mut chunk: Chunk) -> InterpretResult {
        loop {
            let instr = chunk.read_instruction();
            match instr.code {
                OpCode::Constant(x) => {
                    self.push(x);
                }
                OpCode::Add => {
                    binary_op!(self, +);
                },
                OpCode::Subtract => {
                    binary_op!(self, -);
                }
                OpCode::Multiply => {
                    binary_op!(self, *);
                }
                OpCode::Divide => {
                    binary_op!(self, /);
                }
                OpCode::Negate => {
                    let value = self.pop();
                    self.stack.push(-value);
                }
                OpCode::Return => {
                    if let Some(x) = self.stack.pop() {
                        println!("{x}")
                    }
                    return InterpretResult::Ok
                }
            }
        }
    }

    fn pop(&mut self) -> f32 {
        self.stack.pop().expect("VM stack was empty")
    }

    fn push(&mut self, value: f32) {
        self.stack.push(value);
    }
}
