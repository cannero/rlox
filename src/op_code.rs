use serde::{Serialize, Deserialize};

use crate::value::Function;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum OpCode {
    Constant(f64),
    Bool(bool),
    String(String),
    Function(Function),
    Pop,
    GetLocal(usize),
    SetLocal(usize),
    GetGlobal(String),
    DefineGlobal(String),
    SetGlobal(String),
    Equal,
    Greater,
    Less,
    Nil,
    Add,
    Subtract,
    Multiply,
    Divide,
    Not,
    Negate,
    Print,
    Jump(usize),
    JumpIfFalse(usize),
    Loop(usize),
    Call(usize),
    Return,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Instruction {
    pub code: OpCode,
    pub line: i32,
}
