use std::fs::File;
use std::io::Read;
use std::ops::{Index, IndexMut};
use std::path::Path;

pub const MEMORY_SIZE: usize = 1 << 15;

pub struct Memory([u16; MEMORY_SIZE]);

impl Memory {
    pub fn new() -> Memory {
        Memory([0; MEMORY_SIZE])
    }
}

impl Index<usize> for Memory {
    type Output = u16;

    fn index(&self, addr: usize) -> &u16 {
        if addr >= MEMORY_SIZE {
            panic!("Read memory access out of bounds! ({:#06x} >= {:#06x})", addr, MEMORY_SIZE);
        }
        &self.0[addr]
    }
}

impl IndexMut<usize> for Memory {
    fn index_mut(&mut self, addr: usize) -> &mut u16 {
        if addr >= MEMORY_SIZE {
            panic!("Write memory access out of bounds! ({:#06x} >= {:#06x})", addr, MEMORY_SIZE);
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
}
