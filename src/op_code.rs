#[derive(Debug, PartialEq)]
pub enum OpCode {
    Constant(f32),
    Bool(bool),
    String(String),
    Pop,
    GetLocal(usize),
    SetLocal(usize),
    GetGlobal(String),
    DefineGlobal(String),
    SetGlobal(String),
    Equal,
    Greater,
    Less,
    Nil,
    Add,
    Subtract,
    Multiply,
    Divide,
    Not,
    Negate,
    Print,
    Return,
}


pub struct Instruction {
    pub code: OpCode,
    pub line: i32,
}
