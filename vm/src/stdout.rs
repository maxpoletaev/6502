use mos6502::mem::Memory;
use mos6502::types::*;
use std::io;

pub struct Stdout {
    out: Box<dyn io::Write>,
    buf: Vec<u8>,
}

impl Stdout {
    pub fn new(out: Box<dyn io::Write>) -> Self {
        Self {
            out,
            buf: Vec::with_capacity(255),
        }
    }
}

impl Memory for Stdout {
    fn read(&self, _addr: Word) -> Byte {
        0
    }

    fn write(&mut self, addr: Word, data: Byte) {
        self.buf.push(data);
        if addr == 0xFF {
            self.out.write(self.buf.as_slice()).unwrap();
            self.out.flush().unwrap();
            self.buf.clear();
        }
    }
}
