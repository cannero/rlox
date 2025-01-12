use chunk::Chunk;
use debug::Debugger;
use op_code::OpCode;

mod chunk;
mod debug;
mod op_code;


fn main() {
    let mut debugger = Debugger::new();
    let mut chunk = Chunk::new();
    chunk.write(OpCode::Return, 2);
    chunk.write(OpCode::Constant(1.23), 3);
    chunk.write(OpCode::Return, 3);
    debugger.disassemble_chunk(&chunk, "test chunk");
}
