use std::collections::HashMap;

use crate::{chunk::Chunk, compiler::compile, debug::Debugger, op_code::OpCode, value::Value};

pub struct VM {
    stack: Vec<Value>,
    current_line: i32,
    globals: HashMap<String, Value>,
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
        match (a,b) {
            (Value::Number(a), Value::Number(b)) => $vm.push((a + b).into()),
            (Value::String(a), Value::String(b)) => $vm.push((a + &b).into()),
            _ => {
                $vm.runtime_error("Operands must be two numbers or two strings");
                return Err(InterpretResult::RuntimeError);
            }
        }
    }};
    ($vm:ident, $op:tt) => {{
        let b = $vm.pop();
        let a = $vm.pop();
        match (a,b) {
            (Value::Number(a), Value::Number(b)) => $vm.push((a $op b).into()),
            _ => {
                $vm.runtime_error("Operands must be numbers");
                return Err(InterpretResult::RuntimeError);
            }
        }
    }};
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: vec![],
            current_line: 0,
            globals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, source: String, debug: bool) -> InterpretResult {
        match compile(source, debug) {
            Ok(chunk) => {
                if debug {
                    let mut debugger = Debugger::new();
                    debugger.disassemble_chunk(&chunk, "code");
                }
                match self.run(chunk) {
                    Ok(()) => InterpretResult::Ok,
                    Err(res) => res,
                }
            }
            Err(_) => InterpretResult::CompileError,
        }
    }

    fn run(&mut self, mut chunk: Chunk) -> Result<(), InterpretResult> {
        loop {
            let instr = chunk.read_instruction().clone();
            self.current_line = instr.line;
            match &instr.code {
                OpCode::Bool(bool_val) => {
                    self.push(Value::Bool(*bool_val));
                }
                OpCode::Constant(x) => {
                    self.push_number(*x);
                }
                OpCode::Add => {
                    binary_op!(self, +);
                }
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
                    if !self.peek(0).is_number() {
                        self.runtime_error("Operand must be a number");
                        return Err(InterpretResult::RuntimeError);
                    }
                    let value = self.pop_number();
                    self.push_number(-value);
                }
                OpCode::Print => println!("{:?}\n", self.pop()),
                OpCode::Jump(offset) => chunk.jump(*offset),
                OpCode::JumpIfFalse(offset) => {
                    if self.is_falsey(self.peek(0)) {
                        chunk.jump(*offset);
                    }
                }
                OpCode::Loop(offset) => chunk.jump_back(*offset),
                OpCode::Return => return Ok(()),
                OpCode::Pop => _ = self.pop(),
                OpCode::GetLocal(slot) => self.push(self.stack[*slot].clone()),
                OpCode::SetLocal(slot) => self.stack[*slot] = self.peek(0),
                OpCode::GetGlobal(name) => match self.globals.get(name) {
                    Some(val) => self.push(val.clone()),
                    None => {
                        self.runtime_error(&format!("Undefined variable '{}'.", name));
                        return Err(InterpretResult::RuntimeError);
                    }
                },
                OpCode::DefineGlobal(name) => {
                    self.globals.insert(name.clone(), self.peek(0));
                    // todo: check if this is needed:
                    // pop after insert as gc can resize globals
                    self.pop();
                }
                OpCode::SetGlobal(name) => {
                    if self.globals.contains_key(name) {
                        self.globals.insert(name.clone(), self.peek(0));
                    } else {
                        self.runtime_error(&format!("Undefined variable '{}'.", name));
                        return Err(InterpretResult::RuntimeError);
                    }
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
                }
                OpCode::String(string) => {
                    self.push(Value::String(string.clone()));
                }
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
            (Value::String(a), Value::String(b)) => a == b,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn fill_and_run_vm(opcodes: Vec<OpCode>) -> VM {
        let mut vm = VM::new();
        let mut chunk = Chunk::new();
        for code in opcodes {
            chunk.write(code, 1);
        }
        vm.run(chunk).unwrap();
        vm
    }

    #[test]
    fn test_arithmetic() {
        let vm = fill_and_run_vm(vec![
            OpCode::Constant(4.0),
            OpCode::Negate,
            OpCode::Constant(2.0),
            OpCode::Add,
            OpCode::Constant(4.0),
            OpCode::Negate,
            OpCode::Constant(3.0),
            OpCode::Multiply,
            OpCode::Subtract,
            OpCode::Return,
        ]);
        assert_eq!(vm.stack[0], Value::Number(10.0));
    }

    #[test]
    fn test_bool() {
        let mut vm = VM::new();
        let mut chunk = Chunk::new();
        chunk.write2(OpCode::Constant(5.0), OpCode::Constant(4.0), 1);
        chunk.write2(OpCode::Subtract, OpCode::Constant(3.0), 1);
        chunk.write2(OpCode::Constant(2.0), OpCode::Multiply, 1);
        chunk.write2(OpCode::Greater, OpCode::Nil, 1);
        chunk.write2(OpCode::Not, OpCode::Equal, 1);
        chunk.write2(OpCode::Not, OpCode::Return, 1);
        vm.run(chunk).unwrap();
        assert_eq!(vm.stack[0], Value::Bool(true));
    }

    #[test]
    fn test_string() {
        let mut vm = VM::new();
        let mut chunk = Chunk::new();
        chunk.write2(
            OpCode::String("hello".to_string()),
            OpCode::String("world".to_string()),
            1,
        );
        chunk.write2(OpCode::Add, OpCode::Return, 1);
        vm.run(chunk).unwrap();
        assert_eq!(vm.stack[0], Value::String("helloworld".to_string()));
    }

    #[test]
    fn test_set_global() {
        let vm = fill_and_run_vm(vec![
            OpCode::Nil,
            OpCode::DefineGlobal("varx".to_string()),
            OpCode::Constant(1.23),
            OpCode::SetGlobal("varx".to_string()),
            OpCode::Return,
        ]);
        assert_eq!(vm.globals.get("varx").unwrap(), &Value::Number(1.23));
    }
}
