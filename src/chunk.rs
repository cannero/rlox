use crate::op_code::{Instruction, OpCode};

pub trait OpCodeVisitor {
    fn operate(&mut self, code: &OpCode, line: i32);
}

pub struct Chunk {
    instructions: Vec<Instruction>,
    ip: usize,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            instructions: vec![],
            ip: 0,
        }
    }

    pub fn write(&mut self, code: OpCode, line: i32) {
        self.instructions.push(Instruction { code, line });
    }

    #[cfg(test)]
    pub fn write2(&mut self, code: OpCode, code2: OpCode, line: i32) {
        self.write(code, line);
        self.write(code2, line);
    }

    pub fn operate_on_codes(&self, op: &mut dyn OpCodeVisitor) {
        for Instruction { code, line } in &self.instructions {
            op.operate(code, *line);
        }
    }

    pub fn read_instruction(&mut self) -> &Instruction {
        let instr = &self.instructions[self.ip];
        self.ip += 1;
        instr
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

    pub fn jump(&mut self, offset: usize) {
        self.ip += offset;
    }

    pub fn jump_back(&mut self, offset: usize) {
        println!("jumping from {} back {}", self.ip, offset);
        self.ip -= offset;
    }

    pub fn current_offset(&self) -> usize {
        self.instructions.len() - 1
    }
}
