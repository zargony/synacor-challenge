#[macro_use]
extern crate log;
extern crate env_logger;

mod memory;
mod vm;

use memory::Memory;
use vm::VM;

fn main() {
    env_logger::init().unwrap();

    let mem = Memory::challenge_bin();
    let mut vm = VM::new(mem);
    vm.run();
}
