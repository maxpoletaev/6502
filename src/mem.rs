use crate::bus::Device;
use crate::types::*;

const MEM_SIZE: usize = 64 * 1024;

pub struct Memory {
    ram: [Byte; MEM_SIZE],
}

impl Memory {
    pub fn new() -> Memory {
        Memory { ram: [0; MEM_SIZE] }
    }
}

impl Device for Memory {
    fn read(&self, addr: Word) -> Byte {
        self.ram[addr as usize]
    }

    fn write(&mut self, addr: Word, data: Byte) {
        self.ram[addr as usize] = data;
    }
}
