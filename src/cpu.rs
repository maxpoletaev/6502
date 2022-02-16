use crate::mem::Memory;
use crate::opcodes::*;
use crate::types::*;

const FLAG_CARRY: Flag = 1 << 0;
const FLAG_ZERO: Flag = 1 << 1;
const FLAG_NO_INTERRUPT: Flag = 1 << 2;
const FLAG_DECIMAL: Flag = 1 << 3;
const FLAG_BREAK: Flag = 1 << 4;
const FLAG_OVERFLOW: Flag = 1 << 6;
const FLAG_NEGATIVE: Flag = 1 << 7;

pub struct CPU {
    flags: Byte, // Status flags
    sp: Byte,    // Stack pointer
    pc: Word,    // Program counter

    a: Byte, // Accumulator
    x: Byte, // X register
    y: Byte, // Y register

    cycles: u8, // Cycles remaining
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            flags: 0,
            sp: 0,
            pc: 0,
            a: 0,
            x: 0,
            y: 0,
            cycles: 0,
        }
    }

    fn set_flag(&mut self, flag: Flag, value: bool) {
        if value {
            self.flags |= flag;
        } else {
            self.flags &= !flag;
        }
    }

    fn read_flag(&mut self, flag: Flag) -> bool {
        (self.flags & flag) > 0
    }

    fn fetch(&mut self, mem: &mut dyn Memory) -> Byte {
        let data = mem.read(self.pc);
        self.pc += 1;
        data
    }

    fn fetch_word(&mut self, mem: &mut dyn Memory) -> Word {
        let hi = mem.read(self.pc) as Word;
        self.pc += 1;
        let lo = mem.read(self.pc) as Word;
        self.pc += 1;
        (hi << 8) | lo
    }

    fn run_opcode(&mut self, opcode: Byte, mem: &mut dyn Memory) -> u8 {
        match opcode {
            OP_LDA_IMM => self.lda_imm(mem),
            OP_LDA_ZP => self.lda_zp(mem),
            OP_LDA_ZPX => self.lda_zpx(mem),
            OP_LDA_ABS => self.lda_abs(mem),
            OP_LDA_IDX => self.lda_idx(mem),
            OP_LDA_IDY => self.lda_idy(mem),

            OP_INC_ABS => self.inc_abs(mem),
            OP_NOP => self.nop(),

            _ => panic!("unknown opcode: {:02X}", opcode),
        }
    }

    fn nop(&mut self) -> u8 {
        2
    }

    fn set_zn(&mut self) {
        self.set_flag(FLAG_ZERO, self.a == 0x00);
        self.set_flag(FLAG_NEGATIVE, self.a & (1 << 7) > 0); // set if bit 7 of A is set
    }

    fn lda_imm(&mut self, mem: &mut dyn Memory) -> u8 {
        self.a = self.fetch(mem);
        self.set_zn();
        2
    }

    fn lda_zp(&mut self, mem: &mut dyn Memory) -> u8 {
        let zp_addr = self.fetch(mem) as Word;
        self.a = mem.read(zp_addr);
        self.set_zn();
        3
    }

    fn lda_zpx(&mut self, mem: &mut dyn Memory) -> u8 {
        let addr = self.fetch(mem).overflowing_add(self.x).0;
        self.a = mem.read(addr as Word);
        self.set_zn();
        4
    }

    fn lda_abs(&mut self, mem: &mut dyn Memory) -> u8 {
        let addr = self.fetch_word(mem);
        self.a = mem.read(addr);
        self.set_zn();
        4
    }

    fn lda_idx(&mut self, mem: &mut dyn Memory) -> u8 {
        let ptr_addr = {
            let mut addr = self.fetch(mem);
            addr = addr.overflowing_add(self.x).0;
            addr as Word
        };

        let val_addr = {
            let hi = mem.read(ptr_addr) as Word;
            let lo = mem.read(ptr_addr + 1) as Word;
            (hi << 8) | lo
        };

        self.a = mem.read(val_addr);
        self.set_zn();

        6
    }

    fn lda_idy(&mut self, mem: &mut dyn Memory) -> u8 {
        let ptr_addr = self.fetch(mem) as Word;

        let mut val_addr = {
            let hi = mem.read(ptr_addr) as Word;
            let lo = mem.read(ptr_addr.overflowing_add(1).0) as Word;
            (hi << 8) | lo
        };

        let mut cycles = 5;
        let page = val_addr & 0xFF00;
        val_addr = val_addr.overflowing_add(self.y as Word).0;

        if val_addr & 0xFF00 != page {
            // one extra cycle on page cross
            cycles += 1;
        }

        self.a = mem.read(val_addr);
        self.set_zn();

        cycles
    }

    fn inc_abs(&mut self, mem: &mut dyn Memory) -> u8 {
        let addr = self.fetch_word(mem);
        let mut val = mem.read(addr);
        val = val.overflowing_add(1).0;
        mem.write(addr, val);
        self.set_zn();

        6
    }

    pub fn tick(&mut self, mem: &mut dyn Memory) -> bool {
        if self.cycles > 0 {
            // Since we executed the opcode in one go,
            // we just do nothing for the remaining cycles.
            self.cycles -= 1;
            return false;
        }

        let opcode = self.fetch(mem);
        self.cycles = self.run_opcode(opcode, mem);
        self.cycles -= 1;
        true
    }
}

pub fn print_state(cpu: &CPU) {
    println!("--- tick ---");
    println!("FL: 0b{:08b}", cpu.flags);
    println!("PC: 0x{:04X}", cpu.pc);
    println!("SP: 0x{:02X}", cpu.sp);
    println!("A:  0x{:02X}", cpu.a);
    println!("X:  0x{:02X}", cpu.x);
    println!("Y:  0x{:02X}", cpu.y);
}

#[cfg(test)]
#[path = "cpu_test.rs"]
mod cpu_test;
