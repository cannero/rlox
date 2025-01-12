use crate::op_code::OpCode;

pub trait OpCodeVisitor {
    fn operate(&mut self, code: &OpCode, line: usize);
}

struct Instruction {
    code: OpCode,
    line: usize,
}

pub struct Chunk {
    instructions: Vec<Instruction>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {instructions: vec![]}
    }

    pub fn write(&mut self, code: OpCode, line: usize) {
        self.instructions.push(
            Instruction { code, line}
        );
    }

    pub fn operate_on_codes(&self, op: &mut dyn OpCodeVisitor) {
        for Instruction{code, line} in &self.instructions {
            op.operate(code, *line);
        }
    }
}
