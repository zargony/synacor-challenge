use std::fmt;
use std::io::{self, Read};
use super::memory::{MEMORY_SIZE, Memory, Pointer};

pub trait FromPointer: Sized {
    fn from_pointer(ptr: &mut Pointer) -> Option<Self>;
}


#[derive(PartialEq, Eq)]
pub enum Operand {
    Literal(u16),
    Register(u8),
}

impl Operand {
    pub fn get(&self, vm: &VM) -> u16 {
        match *self {
            Operand::Literal(n) => n,
            Operand::Register(r) => vm.reg[r as usize],
        }
    }

    pub fn set(&self, vm: &mut VM, value: u16) {
        match *self {
            Operand::Literal(_) => panic!("Invalid write to literal operand"),
            Operand::Register(r) => vm.reg[r as usize] = value,
        }
    }
}

impl fmt::Debug for Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Operand::Literal(n) => f.write_fmt(format_args!("{:#x}", n)),
            Operand::Register(r) => f.write_fmt(format_args!("R{:x}", r)),
        }
    }
}

impl From<u16> for Operand {
    fn from(n: u16) -> Operand {
        if (n as usize) < MEMORY_SIZE {
            Operand::Literal(n)
        } else if (n as usize) - MEMORY_SIZE < NUM_REGISTERS {
            Operand::Register(((n as usize) - MEMORY_SIZE) as u8)
        } else {
            panic!("Invalid operand {:#06x}", n);
        }
    }
}

impl FromPointer for Operand {
    fn from_pointer(ptr: &mut Pointer) -> Option<Operand> {
        ptr.next().map(|&n| Operand::from(n))
    }
}

impl FromPointer for (Operand, Operand) {
    fn from_pointer(ptr: &mut Pointer) -> Option<(Operand, Operand)> {
        Operand::from_pointer(ptr).and_then(|a|
            Operand::from_pointer(ptr).map(|b|
                (a, b)
            )
        )
    }
}

impl FromPointer for (Operand, Operand, Operand) {
    fn from_pointer(ptr: &mut Pointer) -> Option<(Operand, Operand, Operand)> {
        Operand::from_pointer(ptr).and_then(|a|
            Operand::from_pointer(ptr).and_then(|b|
                Operand::from_pointer(ptr).map(|c|
                    (a, b, c)
                )
            )
        )
    }
}


#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    Halt,
    Set(Operand, Operand),
    Push(Operand),
    Pop(Operand),
    Eq(Operand, Operand, Operand),
    Gt(Operand, Operand, Operand),
    Jmp(Operand),
    Jt(Operand, Operand),
    Jf(Operand, Operand),
    Add(Operand, Operand, Operand),
    Mult(Operand, Operand, Operand),
    Mod(Operand, Operand, Operand),
    And(Operand, Operand, Operand),
    Or(Operand, Operand, Operand),
    Not(Operand, Operand),
    RMem(Operand, Operand),
    WMem(Operand, Operand),
    Call(Operand),
    Ret,
    Out(Operand),
    In(Operand),
    Noop,
}

impl Instruction {
    fn execute(&self, vm: &mut VM) {
        match *self {
            Instruction::Halt => {
                vm.halted = true
            },
            Instruction::Set(ref a, ref b) => {
                let val = b.get(vm);
                a.set(vm, val)
            },
            Instruction::Push(ref a) => {
                let val = a.get(vm);
                vm.stack.push(val);
            },
            Instruction::Pop(ref a) => {
                match vm.stack.pop() {
                    Some(val) => a.set(vm, val),
                    None => panic!("Stack underflow"),
                }
            },
            Instruction::Eq(ref a, ref b, ref c) => {
                match b.get(vm) == c.get(vm) {
                    false => a.set(vm, 0),
                    true => a.set(vm, 1),
                }
            },
            Instruction::Gt(ref a, ref b, ref c) => {
                match b.get(vm) > c.get(vm) {
                    false => a.set(vm, 0),
                    true => a.set(vm, 1),
                }
            },
            Instruction::Jmp(ref a) => {
                vm.ip = a.get(vm) as usize
            },
            Instruction::Jt(ref a, ref b) => {
                if a.get(vm) != 0 {
                    vm.ip = b.get(vm) as usize;
                }
            },
            Instruction::Jf(ref a, ref b) => {
                if a.get(vm) == 0 {
                    vm.ip = b.get(vm) as usize;
                }
            },
            Instruction::Add(ref a, ref b, ref c) => {
                let val = (b.get(vm) + c.get(vm)) % 0x8000;
                a.set(vm, val);
            },
            Instruction::Mult(ref a, ref b, ref c) => {
                let val = ((b.get(vm) as u32 * c.get(vm) as u32) % 0x8000) as u16;
                a.set(vm, val);
            },
            Instruction::Mod(ref a, ref b, ref c) => {
                let val = b.get(vm) % c.get(vm);
                a.set(vm, val);
            },
            Instruction::And(ref a, ref b, ref c) => {
                let val = b.get(vm) & c.get(vm);
                a.set(vm, val);
            },
            Instruction::Or(ref a, ref b, ref c) => {
                let val = b.get(vm) | c.get(vm);
                a.set(vm, val);
            },
            Instruction::Not(ref a, ref b) => {
                let val = !b.get(vm) & 0x7fff;
                a.set(vm, val);
            },
            Instruction::RMem(ref a, ref b) => {
                let addr = b.get(vm) as usize;
                let val = vm.mem[addr];
                a.set(vm, val);
            },
            Instruction::WMem(ref a, ref b) => {
                let addr = a.get(vm) as usize;
                let val = b.get(vm);
                vm.mem[addr] = val;
            },
            Instruction::Call(ref a) => {
                vm.stack.push(vm.ip as u16);
                vm.ip = a.get(vm) as usize;
            },
            Instruction::Ret => {
                match vm.stack.pop() {
                    Some(addr) => vm.ip = addr as usize,
                    None => vm.halted = true,
                }
            },
            Instruction::Out(ref ch) => {
                print!("{}", ch.get(vm) as u8 as char);
            },
            Instruction::In(ref a) => {
                match io::stdin().bytes().next() {
                    Some(Ok(ch)) => a.set(vm, ch as u16),
                    Some(Err(e)) => panic!("Error reading input: {}", e),
                    None => panic!("Input channel closed"),
                }
            },
            Instruction::Noop => (),
        }
    }
}

impl FromPointer for Instruction {
    fn from_pointer(ptr: &mut Pointer) -> Option<Instruction> {
        ptr.next().and_then(|&n| match n {
            0 => Some(Instruction::Halt),
            1 => FromPointer::from_pointer(ptr).map(|(a, b)| Instruction::Set(a, b)),
            2 => FromPointer::from_pointer(ptr).map(|a| Instruction::Push(a)),
            3 => FromPointer::from_pointer(ptr).map(|a| Instruction::Pop(a)),
            4 => FromPointer::from_pointer(ptr).map(|(a, b, c)| Instruction::Eq(a, b, c)),
            5 => FromPointer::from_pointer(ptr).map(|(a, b, c)| Instruction::Gt(a, b, c)),
            6 => FromPointer::from_pointer(ptr).map(|a| Instruction::Jmp(a)),
            7 => FromPointer::from_pointer(ptr).map(|(a, b)| Instruction::Jt(a, b)),
            8 => FromPointer::from_pointer(ptr).map(|(a, b)| Instruction::Jf(a, b)),
            9 => FromPointer::from_pointer(ptr).map(|(a, b, c)| Instruction::Add(a, b, c)),
            10 => FromPointer::from_pointer(ptr).map(|(a, b, c)| Instruction::Mult(a, b, c)),
            11 => FromPointer::from_pointer(ptr).map(|(a, b, c)| Instruction::Mod(a, b, c)),
            12 => FromPointer::from_pointer(ptr).map(|(a, b, c)| Instruction::And(a, b, c)),
            13 => FromPointer::from_pointer(ptr).map(|(a, b, c)| Instruction::Or(a, b, c)),
            14 => FromPointer::from_pointer(ptr).map(|(a, b)| Instruction::Not(a, b)),
            15 => FromPointer::from_pointer(ptr).map(|(a, b)| Instruction::RMem(a, b)),
            16 => FromPointer::from_pointer(ptr).map(|(a, b)| Instruction::WMem(a, b)),
            17 => FromPointer::from_pointer(ptr).map(|a| Instruction::Call(a)),
            18 => Some(Instruction::Ret),
            19 => FromPointer::from_pointer(ptr).map(|a| Instruction::Out(a)),
            20 => FromPointer::from_pointer(ptr).map(|a| Instruction::In(a)),
            21 => Some(Instruction::Noop),
            _ => panic!("Invalid instruction {:#06x}", n),
        })
    }
}


#[derive(Debug)]
pub struct VM {
    mem: Memory,
    reg: [u16; NUM_REGISTERS],
    stack: Vec<u16>,
    ip: usize,
    halted: bool,
}

pub const NUM_REGISTERS: usize = 8;

impl VM {
    pub fn new(mem: Memory) -> VM {
        VM { mem: mem, reg: [0; 8], stack: Vec::new(), ip: 0, halted: false }
    }

    fn next(&mut self) -> Option<Instruction> {
        // XXX: Should keep ptr as a member variable, but Rust doesn't allow self-referencing structs
        let mut ptr = self.mem.pointer(self.ip);
        let ins = Instruction::from_pointer(&mut ptr);
        self.ip = ptr.addr();
        ins
    }

    pub fn step(&mut self) {
        if self.halted { return }
        let addr = self.ip;
        match self.next() {
            Some(instruction) => {
                debug!("{:#06x} {:?}", addr, instruction);
                instruction.execute(self);
            },
            None => panic!("No instruction to execute"),
        }
    }

    pub fn run(&mut self) {
        while !self.halted {
            self.step();
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use memory::Memory;

    #[test]
    fn operands() {
        assert_eq!(Operand::from(0), Operand::Literal(0));
        assert_eq!(Operand::from(32767), Operand::Literal(32767));
        assert_eq!(Operand::from(32768), Operand::Register(0));
        assert_eq!(Operand::from(32775), Operand::Register(7));
    }

    #[test]
    fn operand_fetching() {
        let mem = Memory::from(&[0x1234_u16, 0x5678, 0x8005][..]);
        let mut ptr = mem.pointer(0);
        assert_eq!(Operand::from_pointer(&mut ptr), Some(Operand::Literal(0x1234)));
        assert_eq!(Operand::from_pointer(&mut ptr), Some(Operand::Literal(0x5678)));
        assert_eq!(Operand::from_pointer(&mut ptr), Some(Operand::Register(5)));
    }

    #[test]
    fn instruction_fetching() {
        let mem = Memory::from(&[9_u16, 32768, 32769, 4, 19, 32768][..]);
        let mut ptr = mem.pointer(0);
        assert_eq!(Instruction::from_pointer(&mut ptr), Some(Instruction::Add(Operand::Register(0), Operand::Register(1), Operand::Literal(4))));
        assert_eq!(Instruction::from_pointer(&mut ptr), Some(Instruction::Out(Operand::Register(0))));
    }

    #[test]
    fn vm() {
        let _ = VM::new(Memory::new());
    }
}
