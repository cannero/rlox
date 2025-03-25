use std::{collections::HashMap, time::{SystemTime, UNIX_EPOCH}};

use crate::{compiler::compile, debug::Debugger, op_code::OpCode, value::{Function, NativeFunction, Value}};

struct CallFrame {
    function: Function,
    ip: usize,
    stack_offset: usize,
}

impl CallFrame {
    fn new(function: Function, stack_offset: usize) -> Self {
        Self {
            function,
            ip: 0,
            stack_offset,
        }
    }

    fn increase_ip(&mut self) {
        self.ip += 1;
    }

    fn jump(&mut self, offset: usize) {
        self.ip += offset;
    }

    pub fn jump_back(&mut self, offset: usize) {
        self.ip -= offset;
    }
}

pub struct VM {
    stack: Vec<Value>,
    current_line: i32,
    globals: HashMap<String, Value>,
    frames: Vec<CallFrame>,
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
            (a, b) => {
                $vm.runtime_error(&format!(
                    "Operands must be two numbers or two strings, are {:?} and {:?}",
                    a, b));
                return Err(InterpretResult::RuntimeError);
            }
        }
    }};
    ($vm:ident, $op:tt) => {{
        let b = $vm.pop();
        let a = $vm.pop();
        match (a,b) {
            (Value::Number(a), Value::Number(b)) => $vm.push((a $op b).into()),
            (a, b) => {
                $vm.runtime_error(&format!("Operands must be numbers, are {:?} and {:?}",
                a, b));
                return Err(InterpretResult::RuntimeError);
            }
        }
    }};
}

impl VM {
    pub fn new() -> Self {
        let mut vm = Self {
            stack: vec![],
            current_line: 0,
            globals: HashMap::new(),
            frames: vec![],
        };

        vm.define_natives();
        vm
    }

    pub fn interpret(&mut self, source: String, debug: bool) -> InterpretResult {
        match compile(source, debug) {
            Ok(function) => {
                if debug {
                    let mut debugger = Debugger::new();
                    debugger.disassemble_chunk(&function, "code");
                }

                self.call(function);
                match self.run() {
                    Ok(()) => InterpretResult::Ok,
                    Err(res) => res,
                }
            }
            Err(_) => InterpretResult::CompileError,
        }
    }

    fn run(&mut self) -> Result<(), InterpretResult> {
        loop {
            let frame = self.current_frame();
            let ip = frame.ip;
            frame.increase_ip();

            let instr = frame.function.read_instruction(ip).clone();
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
                OpCode::Jump(offset) => self.current_frame().jump(*offset),
                OpCode::JumpIfFalse(offset) => {
                    if self.is_falsey(self.peek(0)) {
                        self.current_frame().jump(*offset);
                    }
                }
                OpCode::Loop(offset) => self.current_frame().jump_back(*offset),
                OpCode::Call(arg_count) => {
                    if !self.call_value(self.peek(*arg_count), *arg_count) {
                        return Err(InterpretResult::RuntimeError);
                    }
                }
                OpCode::Return => {
                    let result = self.pop();
                    let last_frame = self.frames.pop();
                    if self.frames.is_empty() {
                        // self.pop(); no pop as the first frame is not 'empty'
                        return Ok(());
                    }

                    self.stack.truncate(last_frame.unwrap().stack_offset - 1);
                    self.push(result);
                }
                OpCode::Pop => _ = self.pop(),
                OpCode::GetLocal(slot) => {
                    let stack_offset = self.current_frame().stack_offset;
                    self.push(self.stack[*slot + stack_offset].clone());
                }
                OpCode::SetLocal(slot) => {
                    let stack_offset = self.current_frame().stack_offset;
                    self.stack[*slot + stack_offset] = self.peek(0);
                }
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
                OpCode::Function(fct) => self.push(Value::Function(fct.clone())),
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

    fn call_value(&mut self, value: Value, arg_count: usize) -> bool {
        match value {
            Value::Function(function) => {
                if arg_count != function.arity() {
                    self.runtime_error(&format!(
                        "Expected {} arguments but got {}.",
                        function.arity(), arg_count)
                    );

                    return false;
                }

                self.call(function)
            }
            Value::Native(function, expected_count) => self.call_native(function, expected_count, arg_count),
            _ => {
                self.runtime_error("Can only call functions and classes.");
                false
            }
        }
    }

    fn call(&mut self, function: Function) -> bool {
        let arg_len = function.arity();
        let stack_offset = if self.frames.len() > 2 {
            self.stack.len() - arg_len
        } else {
            self.stack.len() - arg_len
        };

        let frame = CallFrame::new(function, stack_offset);
        self.frames.push(frame);
        true
    }

    fn call_native(&mut self, function: NativeFunction, expected_count: usize, arg_count: usize) -> bool {
        if expected_count != arg_count {
            self.runtime_error(&format!(
                "Expected {} arguments but got {}.",
                expected_count, arg_count)
            );

            return false;
        }

        let mut args = vec![];
        for _ in 0..expected_count {
            args.push(self.pop());
        }

        let result = match function {
            NativeFunction::Clock => {
                let t = SystemTime::now().duration_since(UNIX_EPOCH)
                    .expect("time before unix?")
                    .as_secs_f64();
                Value::Number(t)
            }
        };

        self.pop();
        self.push(result);
        true
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().expect("VM stack was empty")
    }

    fn pop_number(&mut self) -> f64 {
        if let Value::Number(value) = self.pop() {
            value
        } else {
            panic!("pop not a number");
        }
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn push_number(&mut self, value: f64) {
        self.stack.push(Value::Number(value));
    }

    fn current_frame(&mut self) -> &mut CallFrame {
        self.frames.last_mut().expect("frames cannot be empty")
    }

    fn define_natives(&mut self) {
        self.globals.insert("clock".to_string(), Value::Native(NativeFunction::Clock, 0));
    }

    #[allow(dead_code)]
    fn print_stack(&mut self, info: &str) {
        println!("stack, offset {}, {info}", self.current_frame().stack_offset);
        for (i, v) in self.stack.iter().enumerate() {
            match v {
                Value::Function(f) => println!("{i}: Func {}", f.name()),
                o => println!("{i}: {o:?}"),
            }
        }
        println!("");
    }

    fn runtime_error(&self, message: &str) {
        eprintln!("{message}");

        eprintln!("[line {}] in script", self.current_line);
    }
}

#[cfg(test)]
mod tests {
    use crate::chunk::Chunk;

    use super::*;

    fn fill_and_run_vm(opcodes: Vec<OpCode>) -> VM {
        let mut vm = VM::new();
        let mut chunk = Chunk::new();
        for code in opcodes {
            chunk.write(code, 1);
        }
        let function = Function::new_from_chunk("test".to_string(), chunk);
        vm.frames.push(CallFrame::new(function, 0));
        vm.run().unwrap();
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
            OpCode::Nil,
            OpCode::Return,
        ]);
        assert_eq!(vm.stack[0], Value::Number(10.0));
    }

    #[test]
    fn test_bool() {
        let vm = fill_and_run_vm(vec![
            OpCode::Constant(5.0), OpCode::Constant(4.0),
            OpCode::Subtract, OpCode::Constant(3.0),
            OpCode::Constant(2.0), OpCode::Multiply,
            OpCode::Greater, OpCode::Nil,
            OpCode::Not, OpCode::Equal,
            OpCode::Not, OpCode::Nil, OpCode::Return,]);
        assert_eq!(vm.stack[0], Value::Bool(true));
    }

    #[test]
    fn test_string() {
        let vm = fill_and_run_vm(vec![
            OpCode::String("hello".to_string()),
            OpCode::String("world".to_string()),
            OpCode::Add,
            OpCode::Nil,
            OpCode::Return,
        ]);
        assert_eq!(vm.stack[0], Value::String("helloworld".to_string()));
    }

    #[test]
    fn test_set_global() {
        let vm = fill_and_run_vm(vec![
            OpCode::Nil,
            OpCode::DefineGlobal("varx".to_string()),
            OpCode::Constant(1.23),
            OpCode::SetGlobal("varx".to_string()),
            OpCode::Nil,
            OpCode::Return,
        ]);
        assert_eq!(vm.globals.get("varx").unwrap(), &Value::Number(1.23));
    }
}
