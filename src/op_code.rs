#[derive(Debug)]
pub enum OpCode {
    Constant(f32),
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Return,
}


pub struct Instruction {
    pub code: OpCode,
    pub line: i32,
}
