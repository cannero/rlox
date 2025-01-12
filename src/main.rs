use chunk::Chunk;
use debug::disassemble_chunk;
use op_code::OpCode;

mod chunk;
mod debug;
mod op_code;


fn main() {
    let mut chunk = Chunk::new();
    chunk.write(OpCode::Return);
    disassemble_chunk(&chunk, "test chunk");
}
