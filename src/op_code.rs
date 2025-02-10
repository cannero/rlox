#[derive(Debug)]
pub enum OpCode {
    Constant(f32),
    Bool(bool),
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
    Return,
}


pub struct Instruction {
    pub code: OpCode,
    pub line: i32,
}
