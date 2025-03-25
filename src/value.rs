use crate::{chunk::{Chunk, OpCodeVisitor}, op_code::{Instruction, OpCode}};

#[derive(Clone, Debug, PartialEq)]
pub enum NativeFunction {
    Clock,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
    String(String),
    Function(Function),
    Native(NativeFunction, usize),
}

impl Value {
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Self::Number(n)
    }
}

impl From<String> for Value {
    fn from(string: String) -> Self {
        Self::String(string)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    arity: usize,
    name: String,
    chunk: Chunk,
}

impl Function {
    pub fn new(name: String) -> Self {
        Self {
            arity: 0,
            name,
            chunk: Chunk::new(),
        }
    }

    #[cfg(test)]
    pub fn new_from_chunk(name: String, chunk: Chunk) -> Self {
        Self {
            arity: 0,
            name,
            chunk,
        }
    }

    pub fn write(&mut self, code: OpCode, line: i32) {
        self.chunk.write(code, line);
    }

    pub fn current_offset(&self) -> usize {
        self.chunk.current_offset()
    }
    
    pub fn emit_jump(&mut self, code: OpCode, line: i32) -> usize {
        self.chunk.emit_jump(code, line)
    }

    pub fn emit_loop(&mut self, offset: usize, line: i32) {
        self.chunk.emit_loop(offset, line);
    }

    pub fn patch_jump(&mut self, offset: usize) {
        self.chunk.patch_jump(offset);
    }

    pub fn read_instruction(&mut self, ip: usize) -> &Instruction {
        self.chunk.read_instruction(ip)
    }

    pub fn operate_on_codes(&self, op: &mut dyn OpCodeVisitor) {
        self.chunk.operate_on_codes(op);
    }

    pub fn arity(&self) -> usize {
        self.arity
    }

    pub fn increase_arity(&mut self) {
        self.arity += 1;
    }

    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        &self.name
    }
}
