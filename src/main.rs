use chunk::Chunk;
use debug::Debugger;
use op_code::OpCode;
use vm::VM;

mod chunk;
mod debug;
mod op_code;
mod vm;

fn main() {
    let mut debugger = Debugger::new();
    let mut chunk = Chunk::new();
    chunk.write(OpCode::Constant(1.23), 3);
    chunk.write(OpCode::Negate, 3);
    chunk.write(OpCode::Constant(2.89), 3);
    chunk.write(OpCode::Add, 3);
    chunk.write(OpCode::Return, 3);
    debugger.disassemble_chunk(&chunk, "test chunk");

    let mut vm = VM::new(chunk);
    vm.interpret();
}
