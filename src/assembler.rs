use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

/// Set of instruction before labels are calculated.
#[derive(Debug)]
enum PreInstruction {
  SA(isize),
  RB(isize),
  RD,
  WR,
  PRINT,
  SB(isize),
  SF(isize),
  GO(String),
  BIN(String),
  BIZ(String),
  ADD(isize, isize),
  AND(isize, isize),
  MV(isize, isize),
  NOT(isize, isize),
  RS(isize, isize),
  LS(isize, isize),
  SW(isize, isize),
}
/// Set of instruction after labels are calculated.
#[derive(Debug, PartialEq)]
pub enum Instruction {
  SA(isize),
  RB(isize),
  RD,
  WR,
  PRINT,
  SB(isize),
  SF(isize),
  GO(isize),
  BIN(isize),
  BIZ(isize),
  ADD(isize, isize),
  AND(isize, isize),
  MV(isize, isize),
  NOT(isize, isize),
  RS(isize, isize),
  LS(isize, isize),
  SW(isize, isize),
}

lazy_static! {
  pub static ref OP_MAP: HashMap<&'static str, isize> = {
    let mut map = HashMap::new();
    map.insert("SA", 0);
    map.insert("RB", 1);
    map.insert("RD", 2);
    map.insert("WR", 3);
    map.insert("SB", 4);
    map.insert("SF", 5);
    map.insert("LBL", -1);
    map.insert("GO", 6);
    map.insert("BIN", 7);
    map.insert("BIZ", 8);
    map.insert("ADD", 9);
    map.insert("AND", 10);
    map.insert("MV", 11);
    map.insert("NOT", 12);
    map.insert("RS", 13);
    map.insert("LS", 14);
    map.insert("SW", 15);
    map.insert("PRINT", 16);
    map
  };
  pub static ref LABEL_OPS: [isize; 4] = [-1, 6, 7, 8];
  pub static ref ZERO_ARG_OPS: [isize; 3] = [2, 3, 16];
  pub static ref ONE_REG_OPS: [isize; 4] = [0, 1, 4, 5];
  pub static ref TWO_REG_OPS: [isize; 7] = [9, 10, 11, 12, 13, 14, 15];
  pub static ref IS_CNAME: Regex = Regex::new("[_a-zA-Z][_a-zA-Z0-9]*").unwrap();
}

fn print_error(i: usize, line: &str, msg: String) -> ! {
  println!("Error on line #{}: {}", i + 1, msg);
  println!("\t>{}", line);
  std::process::exit(1);
}

fn parse_number(s: &str) -> Result<isize, &'static str> {
  let a = s.strip_prefix("0x");
  if let Some(b) = a {
    match isize::from_str_radix(b, 16) {
      Ok(r) => return Ok(r),
      Err(_) => return Err("hexadecimal"),
    }
  }
  let a = s.strip_prefix("0b");
  if let Some(b) = a {
    match isize::from_str_radix(b, 2) {
      Ok(r) => return Ok(r),
      Err(_) => return Err("binary"),
    }
  }
  match isize::from_str_radix(s, 10) {
    Ok(r) => return Ok(r),
    Err(_) => return Err("character"),
  }
}

#[derive(Debug)]
pub struct Assembly {
  pub reg_inits: Vec<(isize, isize)>,
  pub mem_inits: Vec<(isize, isize)>,
  pub instructions: Vec<Instruction>,
}

impl Assembly {
  pub fn assemble<S: Into<String>>(file: S) -> Self {
    let file = file.into();
    let mut assembly = Assembly {
      reg_inits: vec![],
      mem_inits: vec![],
      instructions: vec![],
    };
    let mut instructions = vec![];
    let mut lbl_lines: HashMap<usize, (usize, &str)> = HashMap::new();
    let mut label_map: HashMap<String, isize> = HashMap::new();
    for (i, line) in file.split("\n").enumerate() {
      let mut code = match line.split_once("#") {
        Some(a) => a.0,
        None => line,
      };
      code = code.trim();
      if code.is_empty() {
        continue;
      }
      code = match code.split_once(";") {
        Some(a) => {
          if !a.1.trim().is_empty() {
            print_error(
              i,
              line,
              format!(
                "Extra non-comment character sequence after semicolon - '{}'",
                a.1
              ),
            );
          }
          a.0
        }
        None => {
          print_error(i, line, "Missing semicolon".to_owned());
        }
      };
      match code.split_once(":") {
        Some((bpart, epart)) => {
          let loc = bpart.trim();
          let val = epart.trim();
          let mut is_reg_init = false;
          let reg = if loc.len() == 1 {
            let r = match isize::from_str_radix(loc, 16) {
              Ok(r) => r,
              Err(_) => print_error(
                i,
                line,
                format!("Invalid register in register initializer - '{}'", loc),
              ),
            };
            is_reg_init = true;
            r
          } else if loc.starts_with("[") && loc.ends_with("]") {
            let mem_str = &loc[1..(loc.len() - 1)];
            match parse_number(mem_str) {
              Ok(r) => r,
              Err(err) => print_error(
                i,
                line,
                format!(
                  "Invalid {} literal in {} initializer - \"{}\"",
                  err, "memory", mem_str
                ),
              ),
            }
          } else {
            print_error(
              i,
              line,
              "Invalid syntax for register/memory initializer".to_owned(),
            );
          };
          let val = match parse_number(val) {
            Ok(r) => r,
            Err(err) => {
              print_error(
                i,
                line,
                format!(
                  "Invalid {} literal in {} initializer - \"{}\"",
                  err, reg, val
                ),
              );
            }
          };
          if is_reg_init {
            assembly.reg_inits.push((reg, val));
          } else {
            assembly.mem_inits.push((reg, val));
          }
          continue;
        }
        None => {}
      }
      let code = code.trim();
      let (op, space, args) = match code.split_once(" ") {
        Some(a) => (a.0, " ", a.1),
        None => (code, "", ""),
      };
      let op = op.to_uppercase();
      if op != "RD" && op != "WR" && op != "PRINT" && space != " " {
        print_error(i, line, format!("Unknown character sequence '{}'", op));
      }
      if !OP_MAP.contains_key(op.as_str()) {
        print_error(i, line, format!("Unknown operation '{}'", op));
      }
      let op_num = *OP_MAP.get(op.as_str()).unwrap();
      let args = args
        .trim()
        .split(",")
        .map(|x| x.trim())
        .filter(|x| !x.is_empty())
        .collect::<Vec<_>>();
      if LABEL_OPS.contains(&op_num) {
        if args.len() > 1 {
          print_error(
            i,
            line,
            format!(
              "Too many arguments for {} operation (expected 1 label, got {} args)",
              op,
              args.len()
            ),
          );
        }
        if args.len() < 1 {
          print_error(
            i,
            line,
            format!(
              "Not enough arguments for {} operation (expected 1 label, got {} args)",
              op,
              args.len()
            ),
          );
        }
        let lbl = args[0].to_owned();
        if op_num == -1 {
          let m = IS_CNAME.is_match(&lbl);
          if !m {
            print_error(
              i,
              line,
              format!("Label name is not a valid cname - '{}'", lbl),
            );
          }
          if label_map.contains_key(&lbl) {
            print_error(i, line, format!("Label '{}' already defined", lbl));
          }
          label_map.insert(lbl, instructions.len() as isize - 1);
          continue;
        }
        lbl_lines.insert(instructions.len(), (i, line));
        let instruction = match op.as_str() {
          "GO" => PreInstruction::GO(lbl),
          "BIN" => PreInstruction::BIN(lbl),
          "BIZ" => PreInstruction::BIZ(lbl),
          _ => unreachable!(),
        };
        instructions.push(instruction);
        continue;
      }
      if ZERO_ARG_OPS.contains(&op_num) {
        if args.len() > 0 {
          print_error(
            i,
            line,
            format!(
              "Too many arguments for {} operation (expected no arguments, got {} args)",
              op,
              args.len()
            ),
          )
        }
        let instruction = match op.as_str() {
          "RD" => PreInstruction::RD,
          "WR" => PreInstruction::WR,
          "PRINT" => PreInstruction::PRINT,
          _ => unreachable!(),
        };
        instructions.push(instruction);
        continue;
      }
      if ONE_REG_OPS.contains(&op_num) {
        if args.len() > 1 {
          print_error(
            i,
            line,
            format!(
              "Too many arguments for {} operation (expected 1 register, got {} args)",
              op,
              args.len()
            ),
          );
        }
        if args.len() < 1 {
          print_error(
            i,
            line,
            format!(
              "Not enough arguments for {} operation (expected 1 register, got {} args)",
              op,
              args.len()
            ),
          );
        }
        let reg = args[0];
        if reg.len() > 1 {
          print_error(i, line, format!("Invalid register specifier '{}'", reg));
        }
        let reg = match isize::from_str_radix(reg, 16) {
          Ok(r) => r,
          Err(_) => {
            print_error(i, line, format!("Invalid register specifier '{}'", reg));
          }
        };
        let instruction = match op.as_str() {
          "SA" => PreInstruction::SA(reg),
          "RB" => PreInstruction::RB(reg),
          "SB" => PreInstruction::SB(reg),
          "SF" => PreInstruction::SF(reg),
          _ => unreachable!(),
        };
        instructions.push(instruction);
        continue;
      }

      if TWO_REG_OPS.contains(&op_num) {
        if args.len() > 2 {
          print_error(
            i,
            line,
            format!(
              "Too many arguments for {} operation (expected 2 registers, got {} args)",
              op,
              args.len()
            ),
          );
        }
        if args.len() < 2 {
          print_error(
            i,
            line,
            format!(
              "Not enough arguments for {} operation (expected 2 registers, got {} args)",
              op,
              args.len()
            ),
          );
        }
        let reg1 = args[0];
        let reg1 = match isize::from_str_radix(reg1, 16) {
          Ok(r) => r,
          Err(_) => {
            print_error(i, line, format!("Invalid register specifier '{}'", reg1));
          }
        };
        let reg2 = args[1];
        let reg2 = match isize::from_str_radix(reg2, 16) {
          Ok(r) => r,
          Err(_) => {
            print_error(i, line, format!("Invalid register specifier '{}'", reg2));
          }
        };
        let instruction = match op.as_str() {
          "ADD" => PreInstruction::ADD(reg1, reg2),
          "AND" => PreInstruction::AND(reg1, reg2),
          "MV" => PreInstruction::MV(reg1, reg2),
          "NOT" => PreInstruction::NOT(reg1, reg2),
          "LS" => PreInstruction::LS(reg1, reg2),
          "RS" => PreInstruction::RS(reg1, reg2),
          "SW" => PreInstruction::SW(reg1, reg2),
          _ => unreachable!(),
        };
        instructions.push(instruction);
        continue;
      }
    }
    let a = instructions
      .iter()
      .enumerate()
      .map(|(j, x)| match x {
        PreInstruction::SA(a) => Instruction::SA(*a),
        PreInstruction::RB(a) => Instruction::RB(*a),
        PreInstruction::RD => Instruction::RD,
        PreInstruction::WR => Instruction::WR,
        PreInstruction::PRINT => Instruction::PRINT,
        PreInstruction::SB(a) => Instruction::SB(*a),
        PreInstruction::SF(a) => Instruction::SF(*a),
        PreInstruction::GO(a) => match label_map.get(a) {
          Some(a) => Instruction::GO(*a),
          None => {
            let (i, line) = lbl_lines.get(&j).unwrap();
            print_error(*i, line, format!("Undefined label reference - '{}'", a))
          }
        },
        PreInstruction::BIN(a) => match label_map.get(a) {
          Some(a) => Instruction::BIN(*a),
          None => {
            let (i, line) = lbl_lines.get(&j).unwrap();
            print_error(*i, line, format!("Undefined label reference - '{}'", a))
          }
        },
        PreInstruction::BIZ(a) => match label_map.get(a) {
          Some(a) => Instruction::BIZ(*a),
          None => {
            let (i, line) = lbl_lines.get(&j).unwrap();
            print_error(*i, line, format!("Undefined label reference - '{}'", a))
          }
        },
        PreInstruction::ADD(a, b) => Instruction::ADD(*a, *b),
        PreInstruction::AND(a, b) => Instruction::AND(*a, *b),
        PreInstruction::MV(a, b) => Instruction::MV(*a, *b),
        PreInstruction::NOT(a, b) => Instruction::NOT(*a, *b),
        PreInstruction::LS(a, b) => Instruction::LS(*a, *b),
        PreInstruction::RS(a, b) => Instruction::RS(*a, *b),
        PreInstruction::SW(a, b) => Instruction::SW(*a, *b),
      })
      .collect::<Vec<_>>();
    assembly.instructions = a;
    assembly
  }
}

#[test]
fn test_comments() {
  let a = Assembly::assemble("#test");
  assert_eq!(a.instructions.len(), 0);
  let b = Assembly::assemble("#test\nADD A, B;");
  assert_eq!(b.instructions.len(), 1);
  let c = Assembly::assemble("ADD A, B; #Test");
  assert_eq!(c.instructions.len(), 1);
  assert!(matches!(c.instructions[0], Instruction::ADD(..)));
}

#[test]
fn test_register_init() {
  let decimal = Assembly::assemble("4: 1024;");
  assert_eq!(decimal.reg_inits[0], (4, 1024));
  let hex = Assembly::assemble("4: 0x1D;");
  assert_eq!(hex.reg_inits[0], (4, 0x1D));
  let binary = Assembly::assemble("4: 0b1010;");
  assert_eq!(binary.reg_inits[0], (4, 0b1010));
}

#[test]
fn test_memory_init() {
  let a = Assembly::assemble("[1024]: 34;");
  assert_eq!(a.mem_inits[0], (1024, 34));
  let b = Assembly::assemble("[0x401]: 0b101;");
  assert_eq!(b.mem_inits[0], (0x401, 0b101));
  let c = Assembly::assemble("[0b10000000010]: 0x10;");
  assert_eq!(c.mem_inits[0], (0b10000000010, 0x10));
  let d = Assembly::assemble("[1056]: 34;");
  assert_eq!(d.mem_inits[0], (1056, 34));
}

#[test]
fn test_instructions() {
  let a = Assembly::assemble("ADD E, A;");
  assert_eq!(a.instructions.len(), 1);
  assert_eq!(a.instructions[0], Instruction::ADD(0xe, 0xa));
  let a = Assembly::assemble("AdD e, A;");
  assert_eq!(a.instructions.len(), 1);
  assert_eq!(a.instructions[0], Instruction::ADD(0xe, 0xa));
  let a = Assembly::assemble("LBL JumpHere;\nADD E, 7;\nSF E;\nBIZ JumpHere;");
  assert_eq!(a.instructions.len(), 3);
  assert_eq!(a.instructions[0], Instruction::ADD(0xe, 0x7));
  assert_eq!(a.instructions[1], Instruction::SF(0xe));
  assert_eq!(a.instructions[2], Instruction::BIZ(-1));
}
