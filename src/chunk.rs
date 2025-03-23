use crate::op_code::{Instruction, OpCode};

pub trait OpCodeVisitor {
    fn operate(&mut self, code: &OpCode, line: i32);
}

#[derive(PartialEq)]
pub struct Chunk {
    instructions: Vec<Instruction>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            instructions: vec![],
        }
    }

    pub fn write(&mut self, code: OpCode, line: i32) {
        self.instructions.push(Instruction { code, line });
    }

    pub fn operate_on_codes(&self, op: &mut dyn OpCodeVisitor) {
        for Instruction { code, line } in &self.instructions {
            op.operate(code, *line);
        }
    }

    pub fn read_instruction(&self, ip: usize) -> &Instruction {
        &self.instructions[ip]
    }

    pub fn emit_jump(&mut self, code: OpCode, line: i32) -> usize {
        self.write(code, line);
        self.current_offset()
    }

    pub fn emit_loop(&mut self, offset: usize, line: i32) {
        self.write(OpCode::Loop(self.current_offset() - offset + 1), line);
    }

    pub fn patch_jump(&mut self, offset: usize) {
        let pos = self.instructions.len() - 1 - offset;
        let new_instruction = match self.instructions.get(offset) {
            Some(Instruction { code, line }) => match code {
                OpCode::JumpIfFalse(_) => Instruction {
                    code: OpCode::JumpIfFalse(pos),
                    line: *line,
                },
                OpCode::Jump(_) => Instruction {
                    code: OpCode::Jump(pos),
                    line: *line,
                },
                other => panic!("Wrong jump patch {:?}", other),
            },
            None => panic!("Invalid jump offset"),
        };

        self.instructions[offset] = new_instruction;
    }

    pub fn current_offset(&self) -> usize {
        self.instructions.len() - 1
    }
}
