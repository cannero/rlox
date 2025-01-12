use crate::op_code::{Instruction, OpCode};

pub trait OpCodeVisitor {
    fn operate(&mut self, code: &OpCode, line: usize);
}

pub struct Chunk {
    instructions: Vec<Instruction>,
    ip: usize,
}

impl Chunk {
    pub fn new() -> Self {
        Self {instructions: vec![], ip: 0}
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

    pub fn read_instruction(&mut self) -> &Instruction {
        let instr = &self.instructions[self.ip];
        self.ip += 1;
        instr
    }
}
