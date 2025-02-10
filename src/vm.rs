use crate::{chunk::Chunk, compiler::compile, debug::Debugger, op_code::OpCode};

#[derive(Clone, Debug)]
enum Value {
    Bool(bool),
    Nil,
    Number(f32),
}

impl Value {
    fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<f32> for Value {
    fn from(n: f32) -> Self {
        Self::Number(n)
    }
}

pub struct VM {
    ip: usize,
    stack: Vec<Value>,
    current_line: i32,
}

#[derive(Debug)]
pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

macro_rules! binary_op {
    ($vm:ident, +) => {{
        let b = $vm.peek(1);
        let a = $vm.peek(0);
        if !a.is_number() || !b.is_number() {
            $vm.runtime_error("Operands must be numbers");
            return InterpretResult::RuntimeError;
        }

        let b = $vm.pop_number();
        let a = $vm.pop_number();
        $vm.push_number(a + b);
    }};
    ($vm:ident, $op:tt) => {{
        let b = $vm.pop();
        let a = $vm.pop();
        match (a,b) {
            (Value::Number(a), Value::Number(b)) => $vm.push((a $op b).into()),
            _ => {
                $vm.runtime_error("Operands must be numbers");
                return InterpretResult::RuntimeError
            }
        }
    }};
}

impl VM {
    
    pub fn new() -> Self {
        Self { ip: 0, stack: vec![], current_line: 0, }
    }

    pub fn interpret(&mut self, source: String, debug: bool) -> InterpretResult {
        match compile(source, debug) {
            Ok(chunk) => {
                if debug {
                    let mut debugger = Debugger::new();
                    debugger.disassemble_chunk(&chunk, "code");
                }
                self.run(chunk)
            },
            Err(_) => InterpretResult::CompileError,
        }
    }

    fn run(&mut self, mut chunk: Chunk) -> InterpretResult {
        loop {
            let instr = chunk.read_instruction();
            self.current_line = instr.line;
            match instr.code {
                OpCode::Bool(bool_val) => {
                    self.push(Value::Bool(bool_val));
                }
                OpCode::Constant(x) => {
                    self.push_number(x);
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
                OpCode::Nil => {
                    self.push(Value::Nil);
                }
                OpCode::Not => {
                    let val = self.pop();
                    self.push(Value::Bool(self.is_falsey(val)));
                }
                OpCode::Negate => {
                    if self.peek(0).is_number() {
                        return InterpretResult::RuntimeError;
                    }
                    let value = self.pop_number();
                    self.push_number(-value);
                }
                OpCode::Return => {
                    if let Some(x) = self.stack.pop() {
                        println!("{x:?}")
                    }
                    return InterpretResult::Ok
                }
                OpCode::Equal => {
                    let b = self.pop();
                    let a = self.pop();
                    
                    self.push(Value::Bool(self.values_equal(a, b)));
                }
                OpCode::Greater => {
                    binary_op!(self, >);
                }
                OpCode::Less => {
                    binary_op!(self, <);
                },
            }
        }
    }


    fn is_falsey(&self, value: Value) -> bool {
        match value {
            Value::Nil => true,
            Value::Bool(val_bool) => !val_bool,
            _ => false,
        }
    }

    fn values_equal(&self, a: Value, b: Value) -> bool {
        match (a, b) {
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::Number(a), Value::Number(b)) => a == b,
            _ => false,
        }
    }

    fn peek(&self, distance: usize) -> Value {
        self.stack[self.stack.len() - 1 - distance].clone()
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().expect("VM stack was empty")
    }

    fn pop_number(&mut self) -> f32 {
        if let Value::Number(value) = self.pop() {
            value
        } else {
            panic!("pop not a number");
        }
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn push_number(&mut self, value: f32) {
        self.stack.push(Value::Number(value));
    }

    fn runtime_error(&self, message: &str) {
        eprintln!("{message}");
        
        eprintln!("[line {}] in script", self.current_line); 
    }
}
