use lazy_static::lazy_static;
use std::sync::RwLock;

use crate::assembler::Instruction;

lazy_static! {
  pub static ref SHOULD_USE_UNSIGNED_INT: RwLock<bool> = RwLock::new(false);
  pub static ref SHOULD_SHOW_BINARY: RwLock<bool> = RwLock::new(false);
}

pub fn op_to_string(op: &Instruction) -> String {
  match op {
    Instruction::GO(a) => format!("GO {:X}", a + 1),
    Instruction::BIN(a) => format!("BIN {:X}", a + 1),
    Instruction::BIZ(a) => format!("BIZ {:X}", a + 1),
    Instruction::ADD(a, b) => format!("ADD {:X}, {:X}", a, b),
    Instruction::AND(a, b) => format!("AND {:X}, {:X}", a, b),
    Instruction::LS(a, b) => format!("LS {:X}, {:X}", a, b),
    Instruction::MV(a, b) => format!("MV {:X}, {:X}", a, b),
    Instruction::NOT(a, b) => format!("NOT {:X}, {:X}", a, b),
    Instruction::RB(a) => format!("RB {:X}", a),
    Instruction::RD => format!("RD"),
    Instruction::RS(a, b) => format!("RS {:X}, {:X}", a, b),
    Instruction::SA(a) => format!("SA {:X}", a),
    Instruction::SB(a) => format!("SB {:X}", a),
    Instruction::SF(a) => format!("SF {:X}", a),
    Instruction::SW(a, b) => format!("SW {:X}, {:X}", a, b),
    Instruction::WR => format!("WR"),
    Instruction::PRINT => format!("PRINT"),
  }
}

pub fn print_code(code: &Vec<Instruction>) {
  for (i, op) in code.iter().enumerate() {
    println!("{:>4}: {}", i, op_to_string(op));
  }
}
