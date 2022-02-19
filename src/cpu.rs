use std::process;

use crate::mem::Memory;
use crate::opcodes::*;
use crate::types::*;

const FL_CARRY: Flag = 1 << 0;
const FL_ZERO: Flag = 1 << 1;
const FL_NO_INTERRUPT: Flag = 1 << 2;
const FL_DECIMAL: Flag = 1 << 3;
const FL_BREAK: Flag = 1 << 4;
const FL_UNUSED: Flag = 1 << 5;
const FL_OVERFLOW: Flag = 1 << 6;
const FL_NEGATIVE: Flag = 1 << 7;

#[derive(Debug)]
enum AddrMode {
    Imm,
    Zp,
    ZpX,
    ZpY,
    Abs,
    AbsX,
    AbsY,
    Ind,
    IndX,
    IndY,
}

#[derive(Debug)]
struct Operand {
    addr: Word,
    value: Byte,
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
            OP_ADC_IMM => self.adc(mem, AddrMode::Imm, 2),
            OP_ADC_ZP0 => self.adc(mem, AddrMode::Zp, 3),
            OP_ADC_ZPX => self.adc(mem, AddrMode::ZpX, 4),
            OP_ADC_ABS => self.adc(mem, AddrMode::Abs, 4),
            OP_ADC_ABX => self.adc(mem, AddrMode::AbsX, 4),
            OP_ADC_ABY => self.adc(mem, AddrMode::AbsY, 4),
            OP_ADC_IDX => self.adc(mem, AddrMode::IndX, 6),
            OP_ADC_IDY => self.adc(mem, AddrMode::IndY, 5),

            OP_LDA_IMM => self.lda(mem, AddrMode::Imm, 2),
            OP_LDA_ZP0 => self.lda(mem, AddrMode::Zp, 3),
            OP_LDA_ZPX => self.lda(mem, AddrMode::ZpX, 4),
            OP_LDA_ABS => self.lda(mem, AddrMode::Abs, 4),
            OP_LDA_ABX => self.lda(mem, AddrMode::AbsX, 4),
            OP_LDA_ABY => self.lda(mem, AddrMode::AbsY, 4),
            OP_LDA_IDX => self.lda(mem, AddrMode::IndX, 6),
            OP_LDA_IDY => self.lda(mem, AddrMode::IndY, 5),

            OP_LDX_IMM => self.ldx(mem, AddrMode::Imm, 2),
            OP_LDX_ZP0 => self.ldx(mem, AddrMode::Zp, 3),
            OP_LDX_ZPY => self.ldx(mem, AddrMode::ZpY, 4),
            OP_LDX_ABS => self.ldx(mem, AddrMode::Abs, 4),
            OP_LDX_ABY => self.ldx(mem, AddrMode::AbsY, 4),

            OP_INC_ZP0 => self.inc(mem, AddrMode::Zp, 5),
            OP_INC_ZPX => self.inc(mem, AddrMode::ZpX, 6),
            OP_INC_ABS => self.inc(mem, AddrMode::Abs, 6),
            OP_INC_ABX => self.inc(mem, AddrMode::AbsX, 7),

            OP_INX_IMP => self.inx(/*mem, AddrMode::Imp,*/ 2),
            OP_INY_IMP => self.iny(/*mem, AddrMode::Imp,*/ 2),

            OP_JMP_ABS => self.jmp(mem, AddrMode::Abs, 3),
            OP_JMP_IND => self.jmp(mem, AddrMode::Ind, 5),

            OP_CLC_IMP => self.clc(/*mem, AddrMode::Imp,*/ 2),

            OP_NOP => self.nop(/*mem, AddrMode::Imp,*/ 2),

            _ => {
                println!("--- last cpu state ---");
                print_state(&self);
                panic!("invalid opcode: 0x{:02x}", opcode);
            }
        }
    }

    fn set_zn(&mut self, data: Byte) {
        self.set_flag(FL_ZERO, data == 0x00);
        self.set_flag(FL_NEGATIVE, data & (1 << 7) > 0); // set if bit 7 of A is set
    }

    fn fetch(&mut self, mem: &mut dyn Memory, mode: AddrMode) -> Operand {
        match mode {
            AddrMode::Imm => {
                let addr = self.pc;
                let value = mem.read(addr);
                self.pc += 1;

                Operand {
                    addr,
                    value,
                    page_cross: false,
                }
            }
            AddrMode::Zp => {
                let addr = mem.read(self.pc) as Word;
                self.pc += 1;

                let value = mem.read(addr);
                Operand {
                    addr,
                    value,
                    page_cross: false,
                }
            }
            AddrMode::ZpX => {
                let zp_addr = mem.read(self.pc);
                self.pc += 1;

                let addr = zp_addr.overflowing_add(self.x).0 as Word;
                let value = mem.read(addr as Word);

                Operand {
                    addr,
                    value,
                    page_cross: false,
                }
            }
            AddrMode::ZpY => {
                let zp_addr = mem.read(self.pc);
                self.pc += 1;

                let addr = zp_addr.overflowing_add(self.y).0 as Word;
                let value = mem.read(addr);

                Operand {
                    addr,
                    value,
                    page_cross: false,
                }
            }
            AddrMode::Abs => {
                let lo = mem.read(self.pc) as Word;
                let hi = mem.read(self.pc + 1) as Word;
                self.pc += 2;

                let addr = (hi << 8) | lo;
                let value = mem.read(addr);

                Operand {
                    addr,
                    value,
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
                let value = mem.read(addr_x);

                Operand {
                    value,
                    page_cross,
                    addr: addr_x,
                }
            }
            AddrMode::AbsY => {
                let lo = mem.read(self.pc) as Word;
                let hi = mem.read(self.pc + 1) as Word;
                self.pc += 2;

                let addr = (hi << 8) | lo;
                let addr_y = addr.overflowing_add(self.y as Word).0;
                let page_cross = addr & 0xFF00 != addr_y & 0xFF00;
                let value = mem.read(addr_y);

                Operand {
                    value,
                    page_cross,
                    addr: addr_y,
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
                    let mut hi = mem.read(ptr_addr + 1) as Word;

                    // An original 6502 has does not correctly fetch the target address if the indirect vector falls on
                    // a page boundary (e.g. $xxFF where xx is any value from $00 to $FF). In this case fetches the LSB
                    // from $xxFF as expected but takes the MSB from $xx00.
                    if ptr_addr & 0x00FF == 0x00FF {
                        hi = mem.read(ptr_addr & 0xFF00) as Word;
                    }

                    (hi << 8) | lo
                };

                let value = mem.read(addr);
                Operand {
                    addr,
                    value,
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

                let value = mem.read(addr);
                Operand {
                    addr,
                    value,
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
                let value = mem.read(addr_y);

                Operand {
                    addr: addr_y,
                    value,
                    page_cross,
                }
            }
        }
    }

    fn nop(&mut self, cycles: u8) -> u8 {
        cycles
    }

    fn clc(&mut self, cycles: u8) -> u8 {
        self.set_flag(FL_CARRY, false);
        cycles
    }

    fn adc(&mut self, mem: &mut dyn Memory, mode: AddrMode, mut cycles: u8) -> u8 {
        let f = self.fetch(mem, mode);

        let (new_a, carry) = self.a.overflowing_add(f.value);
        self.set_flag(FL_CARRY, carry);

        // Detecting signed integer overflow:
        //  1. Check that the accumulator and the operand both have the same sign (bit 7);
        //  2. The overflow happens when the sign changes after the operation.
        let same_sign = (self.a & 1 << 7) ^ (f.value & 1 << 7) == 0;
        let overflow = same_sign && new_a & 1 << 7 != self.a & 1 << 7;
        self.set_flag(FL_OVERFLOW, overflow);

        self.a = new_a;
        self.set_zn(self.a);

        if f.page_cross {
            cycles += 1;
        }
        cycles
    }

    fn lda(&mut self, mem: &mut dyn Memory, mode: AddrMode, mut cycles: u8) -> u8 {
        let f = self.fetch(mem, mode);
        self.a = f.value;
        if f.page_cross {
            cycles += 1;
        }
        self.set_zn(self.a);
        cycles
    }

    fn ldx(&mut self, mem: &mut dyn Memory, mode: AddrMode, mut cycles: u8) -> u8 {
        let f = self.fetch(mem, mode);
        self.x = f.value;
        if f.page_cross {
            cycles += 1;
        }
        self.set_zn(self.x);
        cycles
    }

    fn inc(&mut self, mem: &mut dyn Memory, mode: AddrMode, cycles: u8) -> u8 {
        let f = self.fetch(mem, mode);
        let data = f.value.overflowing_add(1).0;
        mem.write(f.addr, data);
        self.set_zn(data);
        cycles
    }

    fn inx(&mut self, cycles: u8) -> u8 {
        self.x = self.x.overflowing_add(1).0;
        self.set_zn(self.x);
        cycles
    }

    fn iny(&mut self, cycles: u8) -> u8 {
        self.y = self.y.overflowing_add(1).0;
        self.set_zn(self.y);
        cycles
    }

    fn jmp(&mut self, mem: &mut dyn Memory, mode: AddrMode, cycles: u8) -> u8 {
        let f = self.fetch(mem, mode);
        self.pc = f.addr;
        cycles
    }

    pub fn reset(&mut self, reset_vec: Word) {
        self.pc = reset_vec;
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

        self.set_flag(FL_UNUSED, true); // should always be set
        self.cycles = self.run_opcode(opcode, mem);
        true
    }
}

pub fn print_state(cpu: &CPU) {
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
