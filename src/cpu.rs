use crate::mem::Memory;
use crate::opcodes::*;
use crate::types::*;

const FLAG_CARRY: Flag = 1 << 0;
const FLAG_ZERO: Flag = 1 << 1;
const FLAG_NO_INTERRUPT: Flag = 1 << 2;
const FLAG_DECIMAL: Flag = 1 << 3;
const FLAG_BREAK: Flag = 1 << 4;
const FLAG_UNUSED: Flag = 1 << 5;
const FLAG_OVERFLOW: Flag = 1 << 6;
const FLAG_NEGATIVE: Flag = 1 << 7;

#[derive(Debug)]
enum AddrMode {
    Imp,
    Imm,
    Zp,
    ZpX,
    Abs,
    AbsX,
    AbsY,
    Ind,
    IndX,
    IndY,
    Rel,
}

#[derive(Debug)]
struct Fetched {
    addr: Word,
    data: Byte,
    page_cross: bool,
}

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

    fn run_opcode(&mut self, opcode: Byte, mem: &mut dyn Memory) -> u8 {
        match opcode {
            OP_LDA_IMM => self.lda(mem, AddrMode::Imm, 2),
            OP_LDA_ZP0 => self.lda(mem, AddrMode::Zp, 3),
            OP_LDA_ZPX => self.lda(mem, AddrMode::ZpX, 4),
            OP_LDA_ABS => self.lda(mem, AddrMode::Abs, 4),
            OP_LDA_ABX => self.lda(mem, AddrMode::AbsX, 4),
            OP_LDA_ABY => self.lda(mem, AddrMode::AbsY, 4),
            OP_LDA_IDX => self.lda(mem, AddrMode::IndX, 6),
            OP_LDA_IDY => self.lda(mem, AddrMode::IndY, 5),

            OP_INC_ZP0 => self.inc(mem, AddrMode::Zp, 5),
            OP_INC_ZPX => self.inc(mem, AddrMode::ZpX, 6),
            OP_INC_ABS => self.inc(mem, AddrMode::Abs, 6),
            OP_INC_ABX => self.inc(mem, AddrMode::AbsX, 7),

            OP_JMP_ABS => self.jmp(mem, AddrMode::Abs, 3),
            OP_JMP_IND => self.jmp(mem, AddrMode::Ind, 5),

            OP_NOP => self.nop(mem, AddrMode::Imp, 2),

            _ => panic!("unknown opcode: {:02X}", opcode),
        }
    }

    fn nop(&mut self, _mem: &dyn Memory, _mode: AddrMode, cycles: u8) -> u8 {
        cycles
    }

    fn set_zn(&mut self, data: Byte) {
        self.set_flag(FLAG_ZERO, data == 0x00);
        self.set_flag(FLAG_NEGATIVE, data & (1 << 7) > 0); // set if bit 7 of A is set
    }

    fn fetch(&mut self, mem: &mut dyn Memory, mode: AddrMode) -> Fetched {
        match mode {
            AddrMode::Imm => {
                let addr = self.pc;
                let data = mem.read(addr);
                self.pc += 1;

                Fetched {
                    addr,
                    data,
                    page_cross: false,
                }
            }
            AddrMode::Zp => {
                let addr = mem.read(self.pc) as Word;
                self.pc += 1;

                let data = mem.read(addr);
                Fetched {
                    addr,
                    data,
                    page_cross: false,
                }
            }
            AddrMode::ZpX => {
                let mut addr = mem.read(self.pc);
                self.pc += 1;

                addr = addr.overflowing_add(self.x).0;
                let data = mem.read(addr as Word);

                Fetched {
                    addr: (addr as Word),
                    data: data,
                    page_cross: false,
                }
            }
            AddrMode::Abs => {
                let lo = mem.read(self.pc) as Word;
                let hi = mem.read(self.pc + 1) as Word;
                self.pc += 2;

                let addr = (hi << 8) | lo;
                let data = mem.read(addr);

                Fetched {
                    addr,
                    data,
                    page_cross: false,
                }
            }
            AddrMode::AbsX => {
                let lo = mem.read(self.pc) as Word;
                let hi = mem.read(self.pc + 1) as Word;
                self.pc += 2;

                let addr = (hi << 8) | lo;
                let addr_x = addr.overflowing_add(self.x as Word).0;
                let page_cross = addr & 0xFF00 != addr_x & 0xFF00;
                let data = mem.read(addr_x);

                Fetched {
                    addr: addr_x,
                    data,
                    page_cross,
                }
            }
            AddrMode::AbsY => {
                let lo = mem.read(self.pc) as Word;
                let hi = mem.read(self.pc + 1) as Word;
                self.pc += 2;

                let addr = (hi << 8) | lo;
                let addr_y = addr.overflowing_add(self.y as Word).0;
                let page_cross = addr & 0xFF00 != addr_y & 0xFF00;
                let data = mem.read(addr_y);

                Fetched {
                    addr: addr_y,
                    data,
                    page_cross,
                }
            }
            AddrMode::Ind => {
                let ptr_addr = {
                    let lo = mem.read(self.pc) as Word;
                    let hi = mem.read(self.pc + 1) as Word;
                    self.pc += 2;
                    (hi << 8) | lo
                };

                let addr = {
                    let lo = mem.read(ptr_addr) as Word;
                    let hi = mem.read(ptr_addr + 1) as Word;
                    (hi << 8) | lo
                };

                let data = mem.read(addr);
                Fetched {
                    addr,
                    data,
                    page_cross: false,
                }
            }
            AddrMode::IndX => {
                let ptr_addr = {
                    let mut addr = mem.read(self.pc);
                    self.pc += 1;
                    addr = addr.overflowing_add(self.x).0;
                    addr as Word
                };

                let addr = {
                    let lo = mem.read(ptr_addr) as Word;
                    let hi = mem.read(ptr_addr + 1) as Word;
                    (hi << 8) | lo
                };

                let data = mem.read(addr);
                Fetched {
                    addr,
                    data,
                    page_cross: false,
                }
            }
            AddrMode::IndY => {
                let ptr_addr = mem.read(self.pc) as Word;
                self.pc += 1;

                let addr = {
                    let lo = mem.read(ptr_addr) as Word;
                    let hi = mem.read(ptr_addr + 1) as Word;
                    (hi << 8) | lo
                };

                let addr_y = addr.overflowing_add(self.y as Word).0;
                let page_cross = addr & 0xFF00 != addr_y & 0xFF00;
                let data = mem.read(addr_y);

                Fetched {
                    addr: addr_y,
                    data,
                    page_cross,
                }
            }
            _ => panic!("unsupported addressing mode: {:?}", mode),
        }
    }

    fn lda(&mut self, mem: &mut dyn Memory, mode: AddrMode, mut cycles: u8) -> u8 {
        let f = self.fetch(mem, mode);
        self.a = f.data;
        if f.page_cross {
            cycles += 1;
        }
        self.set_zn(self.a);
        cycles
    }

    fn inc(&mut self, mem: &mut dyn Memory, mode: AddrMode, cycles: u8) -> u8 {
        let f = self.fetch(mem, mode);
        let data = f.data.overflowing_add(1).0;
        mem.write(f.addr, data);
        self.set_zn(data);
        cycles
    }

    fn jmp(&mut self, mem: &mut dyn Memory, mode: AddrMode, cycles: u8) -> u8 {
        let f = self.fetch(mem, mode);
        self.pc = f.addr;
        cycles
    }

    pub fn reset(&mut self) {
        self.pc = 0xFFFC;
        self.sp = 0xFF;
        self.flags = 0;

        self.a = 0;
        self.x = 0;
        self.y = 0;

        self.cycles = 0;
    }

    pub fn tick(&mut self, mem: &mut dyn Memory) -> bool {
        if self.cycles > 0 {
            // Since we executed the opcode in one go, we just do nothing for the remaining cycles.
            self.cycles -= 1;
            return false;
        }

        let opcode = mem.read(self.pc);
        self.pc += 1;

        self.cycles = self.run_opcode(opcode, mem);
        self.set_flag(FLAG_UNUSED, true);
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
