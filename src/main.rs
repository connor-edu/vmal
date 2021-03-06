#![feature(str_split_once)]
#![allow(non_snake_case)]

mod assembler;
mod util;
mod vm;

use std::path::PathBuf;
use structopt::StructOpt;
use util::print_code;

#[derive(Debug, StructOpt)]
#[structopt(name = "vmal")]
struct Opt {
  /// Activate debug mode
  #[structopt(short, long)]
  debug: bool,

  /// Input file
  #[structopt(parse(from_os_str))]
  input: PathBuf,

  /// Use unsigned-integers
  #[structopt(short, long)]
  unsigned: bool,

  /// Show binary representation of numbers
  #[structopt(short, long)]
  binary: bool,
}

fn main() {
  let opt = Opt::from_args();
  {
    let mut w = util::SHOULD_USE_UNSIGNED_INT.write().unwrap();
    *w = opt.unsigned;
  }
  {
    let mut w = util::SHOULD_SHOW_BINARY.write().unwrap();
    *w = opt.binary;
  }
  let file = std::fs::read_to_string(opt.input).unwrap();
  let assembly = assembler::Assembly::assemble(file);
  let mut vm = vm::VM::new(assembly.reg_inits, assembly.mem_inits);
  if !opt.debug {
    vm.run_code(&assembly.instructions);
  } else {
    println!("\nAssembled Code:");
    print_code(&assembly.instructions);
    vm.run_debug(&assembly.instructions);
  }
  vm.print_registers();
  if vm.memory.len() > 0 {
    vm.print_memory();
  }
}
