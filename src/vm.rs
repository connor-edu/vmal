use std::collections::{HashMap, HashSet};
use std::io::{stdin, stdout, Write};

use crate::{assembler::Instruction, util::op_to_string};

const INT_MAX: isize = 0xffffffff;
#[derive(Debug)]
pub struct VM {
  pub registers: [isize; 16],
  pub memory: HashMap<isize, isize>,
  pub MAR: isize,
  pub MBR: isize,
  pub N: bool,
  pub Z: bool,
  linecount: usize,
}

fn get_int(reg_val: isize) -> isize {
  if reg_val & (!(INT_MAX >> 1) & INT_MAX) == 0 {
    reg_val
  } else {
    -((-reg_val) & INT_MAX)
  }
}

fn clear_lines(count: usize) {
  let mut stdout = stdout();
  for i in 0..count {
    write!(
      stdout,
      "\u{001B}[2K{}",
      if i < count - 1 { "\u{001B}[1A" } else { "" }
    )
    .unwrap();
  }
  if count > 0 {
    write!(stdout, "\u{001B}[G").unwrap();
  }
  stdout.flush().unwrap();
}

impl VM {
  pub fn new(reg_inits: Vec<(isize, isize)>, mem_inits: Vec<(isize, isize)>) -> Self {
    let mut vm = VM {
      registers: [0; 16],
      memory: HashMap::new(),
      MAR: 0,
      MBR: 0,
      N: false,
      Z: false,
      linecount: 0,
    };

    for (reg, val) in reg_inits {
      vm.registers[reg as usize] = val & INT_MAX;
    }

    vm.registers[0] = 0;
    vm.registers[5] = 0;
    vm.registers[6] = 1;
    vm.registers[7] = INT_MAX;

    for (mem, val) in mem_inits {
      vm.memory.insert(mem, val & INT_MAX);
    }

    vm
  }
  fn set_reg(&mut self, reg: isize, val: isize) {
    self.registers[reg as usize] = val & INT_MAX;
  }
  fn set_mem(&mut self, mem: isize, val: isize) {
    self.memory.insert(mem, val & INT_MAX);
  }
  fn SA(&mut self, x: isize) {
    self.MAR = self.registers[x as usize];
  }
  fn RB(&mut self, x: isize) {
    self.registers[x as usize] = self.MAR;
  }
  fn RD(&mut self) {
    self.MBR = 0;
    match self.memory.get(&self.MAR) {
      Some(a) => {
        self.MBR = *a;
      }
      None => {}
    }
  }
  fn WR(&mut self) {
    self.memory.insert(self.MAR, self.MBR);
  }
  fn SB(&mut self, x: isize) {
    self.MBR = self.registers[x as usize];
  }
  fn SF(&mut self, x: isize) {
    self.Z = self.registers[x as usize] == 0;
    self.N = (self.registers[x as usize] & 0x80000000) == 0;
  }
  fn GO(&mut self, i: isize) {
    self.registers[0] = i;
  }
  fn BIN(&mut self, i: isize) {
    if self.N {
      self.registers[0] = i;
    }
  }
  fn BIZ(&mut self, i: isize) {
    if self.Z {
      self.registers[0] = i;
    }
  }
  fn ADD(&mut self, a: isize, b: isize) {
    self.registers[a as usize] =
      (self.registers[a as usize] + self.registers[b as usize]) & INT_MAX;
  }
  fn AND(&mut self, a: isize, b: isize) {
    self.registers[a as usize] =
      (self.registers[a as usize] & self.registers[b as usize]) & INT_MAX;
  }
  fn MV(&mut self, a: isize, b: isize) {
    self.registers[a as usize] = self.registers[b as usize];
  }
  fn NOT(&mut self, a: isize, b: isize) {
    self.registers[a as usize] = (!self.registers[b as usize]) & INT_MAX;
  }
  fn LS(&mut self, a: isize, b: isize) {
    self.registers[a as usize] = (self.registers[b as usize] << 1) & INT_MAX;
  }
  fn RS(&mut self, a: isize, b: isize) {
    self.registers[a as usize] = (self.registers[b as usize] >> 1) & INT_MAX;
  }
  fn SW(&mut self, a: isize, b: isize) {
    self.MAR = self.registers[a as usize];
    self.MBR = self.registers[b as usize];
    self.memory.insert(self.MAR, self.MBR);
  }
  fn run_op(&mut self, op: &Instruction) {
    match op {
      Instruction::ADD(a, b) => self.ADD(*a, *b),
      Instruction::AND(a, b) => self.AND(*a, *b),
      Instruction::BIN(a) => self.BIN(*a),
      Instruction::BIZ(a) => self.BIZ(*a),
      Instruction::GO(a) => self.GO(*a),
      Instruction::LS(a, b) => self.LS(*a, *b),
      Instruction::MV(a, b) => self.MV(*a, *b),
      Instruction::NOT(a, b) => self.NOT(*a, *b),
      Instruction::RB(a) => self.RB(*a),
      Instruction::RD => self.RD(),
      Instruction::RS(a, b) => self.RS(*a, *b),
      Instruction::SW(a, b) => self.SW(*a, *b),
      Instruction::SA(a) => self.SA(*a),
      Instruction::SB(a) => self.SB(*a),
      Instruction::SF(a) => self.SF(*a),
      Instruction::WR => self.WR(),
    }
  }
  pub fn run_code(&mut self, code: &Vec<Instruction>) {
    while self.registers[0] < code.len() as isize {
      let op = &code[self.registers[0] as usize];
      self.run_op(op);
      self.registers[0] += 1;
    }
  }
  pub fn run_debug(&mut self, code: &Vec<Instruction>) -> bool {
    let stdin = stdin();
    let mut breakpoints: HashSet<isize> = HashSet::new();
    let mut cont = false;
    let mut debug = true;
    while self.registers[0] < code.len() as isize {
      self.linecount = 0;
      let op = &code[self.registers[0] as usize];
      let on_bp = breakpoints.contains(&self.registers[0]);
      if debug && (!cont || on_bp) {
        self.print_registers();

        println!("Flags:");
        println!("  N: {}", self.N);
        println!("  Z: {}", self.Z);
        println!();
        self.linecount += 4;

        if cont {
          println!("Continue till Breakpoint");
          self.linecount += 1;
        }
        if on_bp {
          println!("BREAKPOINT");
          self.linecount += 1;
        }
        println!("Operation: {}", op_to_string(op));
        self.linecount += 1;

        loop {
          let mut s = String::new();
          print!("Debug (n,b,c,r,q): ");
          stdout().flush().unwrap();
          stdin
            .read_line(&mut s)
            .expect("Did not enter a correct string");
          self.linecount += 1;
          let s = s.trim();
          let s = if s.len() == 0 {
            "n".to_owned()
          } else {
            s[0..1].to_lowercase()
          };
          if s == "n" {
          } else if s == "b" {
            print!("Turning Breakpoint ");
            self.linecount += 1;
            if breakpoints.contains(&self.registers[0]) {
              breakpoints.remove(&self.registers[0]);
              println!("OFF");
            } else {
              breakpoints.insert(self.registers[0]);
              println!("ON");
            }
            continue;
          } else if s == "c" {
            cont = !cont;
          } else if s == "r" {
            debug = false;
          } else if s == "q" {
            return false;
          } else {
            panic!();
          }
          break;
        }
        clear_lines(self.linecount + 1);
      }
      self.run_op(&op);
      self.registers[0] += 1;
    }
    println!();
    return true;
  }
  pub fn print_registers(&mut self) {
    println!();
    self.linecount += 1;
    println!("Registers: ");
    self.linecount += 1;
    for (i, v) in self.registers.iter().enumerate() {
      println!("  {:X}: {}", i, get_int(*v));
      self.linecount += 1;
    }
  }
  pub fn print_memory(&mut self) {
    println!();
    println!("Memory:");
    let mut last: Option<isize> = None;
    let mut items = self.memory.iter().collect::<Vec<_>>();
    items.sort_by_key(|x| x.0);
    for (loc, val) in items {
      if let Some(i) = last {
        if loc - i > 1 {
          println!("  ... {} empty locations ...", loc - i - 1);
        }
      }
      println!("  [{}]: {}", loc, get_int(*val));
      last = Some(*loc);
    }
  }
}
