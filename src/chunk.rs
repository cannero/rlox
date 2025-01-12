use crate::op_code::OpCode;

pub struct Chunk {
    codes: Vec<OpCode>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {codes: vec![]}
    }

    pub fn write(&mut self, code: OpCode) {
        self.codes.push(code);
    }

    pub fn operate_on_codes(&self, op: fn(&OpCode)) {
        for code in &self.codes {
            op(code);
        }
    }
}
