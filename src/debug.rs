use crate::{chunk::Chunk, op_code::OpCode};

fn disassemble_instruction(code: &OpCode) {
    println!("{code:?}");
}

pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
    println!("== {} ==", name);

    chunk.operate_on_codes(disassemble_instruction);
}
