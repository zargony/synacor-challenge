use std::fs::File;
use std::io::Read;
use std::ops::{AddAssign, Index, IndexMut, Deref};
use std::path::Path;

pub const MEMORY_SIZE: usize = 1 << 15;
pub const LAST_ADDRESS: usize = MEMORY_SIZE - 1;

pub struct Memory([u16; MEMORY_SIZE]);

impl Memory {
    pub fn new() -> Memory {
        Memory([0; MEMORY_SIZE])
    }
}

impl Index<usize> for Memory {
    type Output = u16;

    fn index(&self, addr: usize) -> &u16 {
        if addr > LAST_ADDRESS {
            panic!("Read memory access out of bounds! ({:#06x} > {:#06x})", addr, LAST_ADDRESS);
        }
        &self.0[addr]
    }
}

impl IndexMut<usize> for Memory {
    fn index_mut(&mut self, addr: usize) -> &mut u16 {
        if addr > LAST_ADDRESS {
            panic!("Write memory access out of bounds! ({:#06x} > {:#06x})", addr, LAST_ADDRESS);
        }
        &mut self.0[addr]
    }
}

impl Memory {
    pub fn load<R: Read>(reader: R) -> Memory {
        let mut mem = Memory::new();
        let mut bytes = reader.bytes();
        for addr in 0.. {
            match bytes.next() {
                Some(Ok(lb)) => match bytes.next() {
                    Some(Ok(hb)) => mem[addr] = (hb as u16) << 8 | (lb as u16),
                    Some(Err(e)) => panic!("Error loading memory: {}", e),
                    None => break,
                },
                Some(Err(e)) => panic!("Error loading memory: {}", e),
                None => break,
            }
        }
        mem
    }

    pub fn load_file<P: AsRef<Path>>(path: P) -> Memory {
        match File::open(path) {
            Ok(file) => Memory::load(file),
            Err(e) => panic!("Error opening file: {}", e),
        }
    }

    pub fn challenge_bin() -> Memory {
        Memory::load_file(Path::new(env!("CARGO_MANIFEST_DIR")).join("challenge").join("challenge.bin"))
    }

    pub fn pointer(&self, addr: usize) -> Pointer {
        Pointer::new(self, addr)
    }
}

pub struct Pointer<'a> {
    mem: &'a Memory,
    addr: usize,
}

impl<'a> Pointer<'a> {
    pub fn new(mem: &Memory, addr: usize) -> Pointer {
        Pointer { mem: mem, addr: addr }
    }

    pub fn jump(&mut self, addr: usize) {
        self.addr = addr;
    }
}

impl<'a>AddAssign<usize> for Pointer<'a> {
    fn add_assign(&mut self, offset: usize) {
        self.addr += offset;
    }
}

impl<'a> Deref for Pointer<'a> {
    type Target = u16;

    fn deref(&self) -> &u16 {
        self.mem.index(self.addr)
    }
}

impl<'a> Iterator for Pointer<'a> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        let value = **self;
        *self += 1;
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_access() {
        let mut mem = Memory::new();
        assert_eq!(mem[123], 0);
        mem[123] = 456;
        assert_eq!(mem[123], 456);
    }

    #[test]
    fn loading() {
        let mem = Memory::load(&[0x12u8, 0x34, 0x56, 0x78, 0x9a, 0xbc][..]);
        assert_eq!(mem[0], 0x3412);
        assert_eq!(mem[1], 0x7856);
        assert_eq!(mem[2], 0xbc9a);
        assert_eq!(mem[3], 0);
    }

    #[test]
    fn loading_file() {
        let mem = Memory::challenge_bin();
        assert_eq!(mem[0], 0x0015);
        assert_eq!(mem[1], 0x0015);
        assert_eq!(mem[2], 0x0013);
        assert_eq!(mem[3], 0x0057);
    }

    #[test]
    fn pointer_operations() {
        let mem = Memory::new();
        let mut ptr = mem.pointer(123);
        assert_eq!(ptr.addr, 123);
        ptr.jump(456);
        assert_eq!(ptr.addr, 456);
        ptr += 111;
        assert_eq!(ptr.addr, 567);
        assert_eq!(*ptr, 0);
        assert_eq!(ptr.next(), Some(0));
        assert_eq!(ptr.addr, 568);
    }
}
