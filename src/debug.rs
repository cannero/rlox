use crate::{
    chunk::OpCodeVisitor,
    op_code::OpCode, value::Function,
};

pub struct Debugger {
    last_line: i32,
}

impl Debugger {
    pub fn new() -> Self {
        Self { last_line: 0 }
    }

    pub fn disassemble_chunk(&mut self, function: &Function, name: &str) {
        println!("== {} ==", name);

        function.operate_on_codes(self);
    }
}

impl OpCodeVisitor for Debugger {
    fn operate(&mut self, code: &OpCode, line: i32) {
        let line_or_placeholder = if line == self.last_line {
            "|".to_string()
        } else {
            line.to_string()
        };

        println!("{line_or_placeholder:>4} {code:?}");
        self.last_line = line;
    }
}
