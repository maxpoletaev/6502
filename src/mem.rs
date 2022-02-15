use crate::types::*;

const RAM_SIZE: usize = 64 * 1024;

pub trait Memory {
    fn read(&self, addr: Word) -> Byte;
    fn write(&mut self, addr: Word, data: Byte);
}

pub struct Ram {
    data: [Byte; RAM_SIZE],
}

impl Ram {
    pub fn new() -> Ram {
        Ram {
            data: [0; RAM_SIZE],
        }
    }
}

impl Memory for Ram {
    fn read(&self, addr: Word) -> Byte {
        self.data[addr as usize]
    }

    fn write(&mut self, addr: Word, data: Byte) {
        self.data[addr as usize] = data;
    }
}
