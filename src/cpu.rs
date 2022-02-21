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

#[derive(Debug, Clone)]
enum AddrMode {
    Acc,
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
    Rel,
}

#[derive(Debug)]
struct Operand {
    addr: Word,
    value: Byte,
    page_cross: bool,
}

pub struct CPU {
    sp: Byte, // Stack pointer
    pc: Word, // Program counter

    p: Byte, // Status flags
    a: Byte, // Accumulator
    x: Byte, // X register
    y: Byte, // Y register

    cycles: u8, // Cycles remaining
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            sp: 0,
            pc: 0,
            p: 0,
            a: 0,
            x: 0,
            y: 0,
            cycles: 0,
        }
    }

    fn set_flag(&mut self, flag: Flag, value: bool) {
        if value {
            self.p |= flag;
        } else {
            self.p &= !flag;
        }
    }

    fn read_flag(&self, flag: Flag) -> bool {
        (self.p & flag) > 0
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

            OP_LDY_IMM => self.ldy(mem, AddrMode::Imm, 2),
            OP_LDY_ZP0 => self.ldy(mem, AddrMode::Zp, 3),
            OP_LDY_ZPX => self.ldy(mem, AddrMode::ZpX, 4),
            OP_LDY_ABS => self.ldy(mem, AddrMode::Abs, 4),
            OP_LDY_ABX => self.ldy(mem, AddrMode::AbsX, 4),

            OP_STA_ZP0 => self.sta(mem, AddrMode::Zp, 3),
            OP_STA_ZPX => self.sta(mem, AddrMode::ZpX, 4),
            OP_STA_ABS => self.sta(mem, AddrMode::Abs, 4),
            OP_STA_ABX => self.sta(mem, AddrMode::AbsX, 5),
            OP_STA_ABY => self.sta(mem, AddrMode::AbsY, 5),
            OP_STA_IDX => self.sta(mem, AddrMode::IndX, 6),
            OP_STA_IDY => self.sta(mem, AddrMode::IndY, 6),

            OP_STX_ZP0 => self.stx(mem, AddrMode::Zp, 3),
            OP_STX_ZPY => self.stx(mem, AddrMode::ZpY, 4),
            OP_STX_ABS => self.stx(mem, AddrMode::Abs, 4),

            OP_STY_ZP0 => self.sty(mem, AddrMode::Zp, 3),
            OP_STY_ZPX => self.sty(mem, AddrMode::ZpX, 4),
            OP_STY_ABS => self.sty(mem, AddrMode::Abs, 4),

            OP_INC_ZP0 => self.inc(mem, AddrMode::Zp, 5),
            OP_INC_ZPX => self.inc(mem, AddrMode::ZpX, 6),
            OP_INC_ABS => self.inc(mem, AddrMode::Abs, 6),
            OP_INC_ABX => self.inc(mem, AddrMode::AbsX, 7),
            OP_INX_IMP => self.inx(/*mem, AddrMode::Imp,*/ 2),
            OP_INY_IMP => self.iny(/*mem, AddrMode::Imp,*/ 2),

            OP_DEC_ZP0 => self.dec(mem, AddrMode::Zp, 5),
            OP_DEC_ZPX => self.dec(mem, AddrMode::ZpX, 6),
            OP_DEC_ABS => self.dec(mem, AddrMode::Abs, 6),
            OP_DEC_ABX => self.dec(mem, AddrMode::AbsX, 7),
            OP_DEX_IMP => self.dex(/*mem, AddrMode::Imp,*/ 2),
            OP_DEY_IMP => self.dey(/*mem, AddrMode::Imp,*/ 2),

            OP_JMP_ABS => self.jmp(mem, AddrMode::Abs, 3),
            OP_JMP_IND => self.jmp(mem, AddrMode::Ind, 5),
            OP_JSR_ABS => self.jsr(mem, AddrMode::Abs, 6),
            OP_RTS_IMP => self.rts(mem, /*AddrMode::Imp,*/ 6),

            OP_BCC_REL => self.bcc(mem, AddrMode::Rel, 2),
            OP_BCS_REL => self.bcs(mem, AddrMode::Rel, 2),
            OP_BEQ_REL => self.beq(mem, AddrMode::Rel, 2),
            OP_BNE_REL => self.bne(mem, AddrMode::Rel, 2),
            OP_BMI_REL => self.bmi(mem, AddrMode::Rel, 2),
            OP_BPL_REL => self.bpl(mem, AddrMode::Rel, 2),
            OP_BVC_REL => self.bvc(mem, AddrMode::Rel, 2),
            OP_BVS_REL => self.bvs(mem, AddrMode::Rel, 2),

            OP_CMP_IMM => self.cmp(mem, AddrMode::Imm, 2),
            OP_CMP_ZP0 => self.cmp(mem, AddrMode::Zp, 3),
            OP_CMP_ZPX => self.cmp(mem, AddrMode::ZpX, 4),
            OP_CMP_ABS => self.cmp(mem, AddrMode::Abs, 4),
            OP_CMP_ABX => self.cmp(mem, AddrMode::AbsX, 4),
            OP_CMP_ABY => self.cmp(mem, AddrMode::AbsY, 4),
            OP_CMP_IDX => self.cmp(mem, AddrMode::IndX, 6),
            OP_CMP_IDY => self.cmp(mem, AddrMode::IndY, 5),

            OP_CPX_IMM => self.cpx(mem, AddrMode::Imm, 2),
            OP_CPX_ZP0 => self.cpx(mem, AddrMode::Zp, 3),
            OP_CPX_ABS => self.cpx(mem, AddrMode::Abs, 4),

            OP_CPY_IMM => self.cpy(mem, AddrMode::Imm, 2),
            OP_CPY_ZP0 => self.cpy(mem, AddrMode::Zp, 3),
            OP_CPY_ABS => self.cpy(mem, AddrMode::Abs, 4),

            OP_CLC_IMP => self.clc(/*mem, AddrMode::Imp,*/ 2),
            OP_CLI_IMP => self.cli(/*mem, AddrMode::Imp,*/ 2),
            OP_CLV_IMP => self.clv(/*mem, AddrMode::Imp,*/ 2),

            OP_SEC_IMP => self.sec(/*mem, AddrMode::Imp,*/ 2),
            OP_SEI_IMP => self.sei(/*mem, AddrMode::Imp,*/ 2),

            OP_TAX_IMP => self.tax(/*mem, AddrMode::Imp,*/ 2),
            OP_TXA_IMP => self.txa(/*mem, AddrMode::Imp,*/ 2),
            OP_TAY_IMP => self.tay(/*mem, AddrMode::Imp,*/ 2),
            OP_TYA_IMP => self.tya(/*mem, AddrMode::Imp,*/ 2),
            OP_TSX_IMP => self.tsx(/*mem, AddrMode::Imp,*/ 2),
            OP_TXS_IMP => self.txs(/*mem, AddrMode::Imp,*/ 2),

            OP_PHA_IMP => self.pha(mem, /*AddrMode::Imp,*/ 3),
            OP_PHP_IMP => self.php(mem, /*AddrMode::Imp,*/ 3),
            OP_PLA_IMP => self.pla(mem, /*AddrMode::Imp,*/ 4),
            OP_PLP_IMP => self.plp(mem, /*AddrMode::Imp,*/ 4),

            OP_AND_IMM => self.and(mem, AddrMode::Imm, 2),
            OP_AND_ZP0 => self.and(mem, AddrMode::Zp, 3),
            OP_AND_ZPX => self.and(mem, AddrMode::ZpX, 4),
            OP_AND_ABS => self.and(mem, AddrMode::Abs, 4),
            OP_AND_ABX => self.and(mem, AddrMode::AbsX, 4),
            OP_AND_ABY => self.and(mem, AddrMode::AbsY, 4),
            OP_AND_IDX => self.and(mem, AddrMode::IndX, 6),
            OP_AND_IDY => self.and(mem, AddrMode::IndY, 5),

            OP_ORA_IMM => self.ora(mem, AddrMode::Imm, 2),
            OP_ORA_ZP0 => self.ora(mem, AddrMode::Zp, 3),
            OP_ORA_ZPX => self.ora(mem, AddrMode::ZpX, 4),
            OP_ORA_ABS => self.ora(mem, AddrMode::Abs, 4),
            OP_ORA_ABX => self.ora(mem, AddrMode::AbsX, 4),
            OP_ORA_ABY => self.ora(mem, AddrMode::AbsY, 4),
            OP_ORA_IDX => self.ora(mem, AddrMode::IndX, 6),
            OP_ORA_IDY => self.ora(mem, AddrMode::IndY, 5),

            OP_BIT_ZP0 => self.bit(mem, AddrMode::Zp, 3),
            OP_BIT_ABS => self.bit(mem, AddrMode::Abs, 4),

            OP_EOR_IMM => self.eor(mem, AddrMode::Imm, 2),
            OP_EOR_ZP0 => self.eor(mem, AddrMode::Zp, 3),
            OP_EOR_ZPX => self.eor(mem, AddrMode::ZpX, 4),
            OP_EOR_ABS => self.eor(mem, AddrMode::Abs, 4),
            OP_EOR_ABX => self.eor(mem, AddrMode::AbsX, 4),
            OP_EOR_ABY => self.eor(mem, AddrMode::AbsY, 4),
            OP_EOR_IDX => self.eor(mem, AddrMode::IndX, 6),
            OP_EOR_IDY => self.eor(mem, AddrMode::IndY, 5),

            OP_ASL_ACC => self.asl(mem, AddrMode::Acc, 2),
            OP_ASL_ZP0 => self.asl(mem, AddrMode::Zp, 5),
            OP_ASL_ZPX => self.asl(mem, AddrMode::ZpX, 6),
            OP_ASL_ABS => self.asl(mem, AddrMode::Abs, 6),
            OP_ASL_ABX => self.asl(mem, AddrMode::AbsX, 7),

            OP_LSR_ACC => self.lsr(mem, AddrMode::Acc, 2),
            OP_LSR_ZP0 => self.lsr(mem, AddrMode::Zp, 5),
            OP_LSR_ZPX => self.lsr(mem, AddrMode::ZpX, 6),
            OP_LSR_ABS => self.lsr(mem, AddrMode::Abs, 6),
            OP_LSR_ABX => self.lsr(mem, AddrMode::AbsX, 7),

            OP_ROL_ACC => self.rol(mem, AddrMode::Acc, 2),
            OP_ROL_ZP0 => self.rol(mem, AddrMode::Zp, 5),
            OP_ROL_ZPX => self.rol(mem, AddrMode::ZpX, 6),
            OP_ROL_ABS => self.rol(mem, AddrMode::Abs, 6),
            OP_ROL_ABX => self.rol(mem, AddrMode::AbsX, 7),

            OP_ROR_ACC => self.ror(mem, AddrMode::Acc, 2),
            OP_ROR_ZP0 => self.ror(mem, AddrMode::Zp, 5),
            OP_ROR_ZPX => self.ror(mem, AddrMode::ZpX, 6),
            OP_ROR_ABS => self.ror(mem, AddrMode::Abs, 6),
            OP_ROR_ABX => self.ror(mem, AddrMode::AbsX, 7),

            OP_NOP => self.nop(/*mem, AddrMode::Imp,*/ 2),

            _ => {
                println!("---- last cpu state ----");
                print_state(&self);
                panic!("invalid opcode: 0x{:02x}", opcode);
            }
        }
    }

    fn set_zn(&mut self, data: Byte) {
        self.set_flag(FL_ZERO, data == 0x00);
        self.set_flag(FL_NEGATIVE, data & (1 << 7) > 0); // set if bit 7 of A is set
    }

    fn stack_push(&mut self, mem: &mut dyn Memory, data: Byte) {
        let addr = 0x0100 | (self.sp as Word);
        self.sp = self.sp.overflowing_sub(1).0;
        mem.write(addr, data);
    }

    fn stack_pop(&mut self, mem: &mut dyn Memory) -> Byte {
        self.sp = self.sp.overflowing_add(1).0;
        let addr = 0x0100 | (self.sp as Word);
        let data = mem.read(addr);
        data
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
            AddrMode::Rel => {
                let mut offset = mem.read(self.pc) as Word;
                self.pc += 1;

                if offset & (1 << 7) != 0 {
                    offset = offset | 0xFF00;
                }

                let addr = self.pc.overflowing_add(offset).0;
                let page_cross = self.pc & 0xFF00 != addr & 0xFF00;
                let value = mem.read(addr);

                Operand {
                    addr,
                    value,
                    page_cross,
                }
            }
            AddrMode::Acc => Operand {
                addr: 0x00,
                value: self.a,
                page_cross: false,
            },
        }
    }

    fn nop(&mut self, cycles: u8) -> u8 {
        cycles
    }

    fn clc(&mut self, cycles: u8) -> u8 {
        self.set_flag(FL_CARRY, false);
        cycles
    }

    fn cli(&mut self, cycles: u8) -> u8 {
        self.set_flag(FL_NO_INTERRUPT, false);
        cycles
    }

    fn clv(&mut self, cycles: u8) -> u8 {
        self.set_flag(FL_OVERFLOW, false);
        cycles
    }

    fn sec(&mut self, cycles: u8) -> u8 {
        self.set_flag(FL_CARRY, true);
        cycles
    }

    fn sei(&mut self, cycles: u8) -> u8 {
        self.set_flag(FL_NO_INTERRUPT, true);
        cycles
    }

    fn adc(&mut self, mem: &mut dyn Memory, mode: AddrMode, mut cycles: u8) -> u8 {
        let f = self.fetch(mem, mode);

        let (new_a, carry) = self.a.overflowing_add(f.value);
        self.set_flag(FL_CARRY, carry);

        // Detecting signed integer overflow:
        //  1. Check that the initial value of the accumulator and the operand both have the same sign (bit 7);
        //  2. We can tell that overflow took place if bit 7 is not the same anymore after the operation.
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

    fn ldy(&mut self, mem: &mut dyn Memory, mode: AddrMode, mut cycles: u8) -> u8 {
        let f = self.fetch(mem, mode);
        self.y = f.value;
        if f.page_cross {
            cycles += 1;
        }
        self.set_zn(self.y);
        cycles
    }

    fn sta(&mut self, mem: &mut dyn Memory, mode: AddrMode, cycles: u8) -> u8 {
        let f = self.fetch(mem, mode);
        mem.write(f.addr, self.a);
        cycles
    }

    fn stx(&mut self, mem: &mut dyn Memory, mode: AddrMode, cycles: u8) -> u8 {
        let f = self.fetch(mem, mode);
        mem.write(f.addr, self.x);
        cycles
    }

    fn sty(&mut self, mem: &mut dyn Memory, mode: AddrMode, cycles: u8) -> u8 {
        let f = self.fetch(mem, mode);
        mem.write(f.addr, self.y);
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

    fn dec(&mut self, mem: &mut dyn Memory, mode: AddrMode, cycles: u8) -> u8 {
        let f = self.fetch(mem, mode);
        let data = f.value.overflowing_sub(1).0;
        mem.write(f.addr, data);
        self.set_zn(data);
        cycles
    }

    fn dex(&mut self, cycles: u8) -> u8 {
        self.x = self.x.overflowing_sub(1).0;
        self.set_zn(self.x);
        cycles
    }

    fn dey(&mut self, cycles: u8) -> u8 {
        self.y = self.y.overflowing_sub(1).0;
        self.set_zn(self.y);
        cycles
    }

    fn jmp(&mut self, mem: &mut dyn Memory, mode: AddrMode, cycles: u8) -> u8 {
        let f = self.fetch(mem, mode);
        self.pc = f.addr;
        cycles
    }

    fn jsr(&mut self, mem: &mut dyn Memory, mode: AddrMode, cycles: u8) -> u8 {
        let f = self.fetch(mem, mode);
        self.stack_push(mem, self.pc as Byte);
        self.stack_push(mem, (self.pc >> 8) as Byte);
        self.pc = f.addr;
        cycles
    }

    fn rts(&mut self, mem: &mut dyn Memory, cycles: u8) -> u8 {
        let hi = self.stack_pop(mem) as Word;
        let lo = self.stack_pop(mem) as Word;
        self.pc = (hi << 8) | lo;
        cycles
    }

    fn bcc(&mut self, mem: &mut dyn Memory, mode: AddrMode, mut cycles: u8) -> u8 {
        let f = self.fetch(mem, mode);
        if !self.read_flag(FL_CARRY) {
            cycles += 2;
            if f.page_cross {
                cycles += 2;
            }
            self.pc = f.addr;
        }
        cycles
    }

    fn bcs(&mut self, mem: &mut dyn Memory, mode: AddrMode, mut cycles: u8) -> u8 {
        let f = self.fetch(mem, mode);
        if self.read_flag(FL_CARRY) {
            cycles += 2;
            if f.page_cross {
                cycles += 2;
            }
            self.pc = f.addr;
        }
        cycles
    }

    fn beq(&mut self, mem: &mut dyn Memory, mode: AddrMode, mut cycles: u8) -> u8 {
        let f = self.fetch(mem, mode);
        if self.read_flag(FL_ZERO) {
            cycles += 2;
            if f.page_cross {
                cycles += 2;
            }
            self.pc = f.addr;
        }
        cycles
    }

    fn bne(&mut self, mem: &mut dyn Memory, mode: AddrMode, mut cycles: u8) -> u8 {
        let f = self.fetch(mem, mode);
        if !self.read_flag(FL_ZERO) {
            cycles += 2;
            if f.page_cross {
                cycles += 2;
            }
            self.pc = f.addr;
        }
        cycles
    }

    fn bmi(&mut self, mem: &mut dyn Memory, mode: AddrMode, mut cycles: u8) -> u8 {
        let f = self.fetch(mem, mode);
        if self.read_flag(FL_NEGATIVE) {
            cycles += 2;
            if f.page_cross {
                cycles += 2;
            }
            self.pc = f.addr;
        }
        cycles
    }

    fn bpl(&mut self, mem: &mut dyn Memory, mode: AddrMode, mut cycles: u8) -> u8 {
        let f = self.fetch(mem, mode);
        if !self.read_flag(FL_NEGATIVE) {
            cycles += 2;
            if f.page_cross {
                cycles += 2;
            }
            self.pc = f.addr;
        }
        cycles
    }

    fn bvc(&mut self, mem: &mut dyn Memory, mode: AddrMode, mut cycles: u8) -> u8 {
        let f = self.fetch(mem, mode);
        if !self.read_flag(FL_OVERFLOW) {
            cycles += 2;
            if f.page_cross {
                cycles += 2;
            }
            self.pc = f.addr;
        }
        cycles
    }

    fn bvs(&mut self, mem: &mut dyn Memory, mode: AddrMode, mut cycles: u8) -> u8 {
        let f = self.fetch(mem, mode);
        if self.read_flag(FL_OVERFLOW) {
            cycles += 2;
            if f.page_cross {
                cycles += 2;
            }
            self.pc = f.addr;
        }
        cycles
    }

    fn cmp(&mut self, mem: &mut dyn Memory, mode: AddrMode, mut cycles: u8) -> u8 {
        let f = self.fetch(mem, mode);
        let r = self.a.overflowing_sub(f.value).0;
        self.set_flag(FL_NEGATIVE, r & (1 << 7) != 0);
        self.set_flag(FL_CARRY, self.a >= f.value);
        self.set_flag(FL_ZERO, r == 0);
        if f.page_cross {
            cycles += 1;
        }
        cycles
    }

    fn cpx(&mut self, mem: &mut dyn Memory, mode: AddrMode, mut cycles: u8) -> u8 {
        let f = self.fetch(mem, mode);
        let r = self.x.overflowing_sub(f.value).0;
        self.set_flag(FL_NEGATIVE, r & (1 << 7) != 0);
        self.set_flag(FL_CARRY, self.x >= f.value);
        self.set_flag(FL_ZERO, r == 0);
        if f.page_cross {
            cycles += 1;
        }
        cycles
    }

    fn cpy(&mut self, mem: &mut dyn Memory, mode: AddrMode, mut cycles: u8) -> u8 {
        let f = self.fetch(mem, mode);
        let r = self.y.overflowing_sub(f.value).0;
        self.set_flag(FL_NEGATIVE, r & (1 << 7) != 0);
        self.set_flag(FL_CARRY, self.y >= f.value);
        self.set_flag(FL_ZERO, r == 0);
        if f.page_cross {
            cycles += 1;
        }
        cycles
    }

    fn tax(&mut self, cycles: u8) -> u8 {
        self.x = self.a;
        self.set_zn(self.x);
        cycles
    }

    fn txa(&mut self, cycles: u8) -> u8 {
        self.a = self.x;
        self.set_zn(self.a);
        cycles
    }

    fn tay(&mut self, cycles: u8) -> u8 {
        self.y = self.a;
        self.set_zn(self.y);
        cycles
    }

    fn tya(&mut self, cycles: u8) -> u8 {
        self.a = self.y;
        self.set_zn(self.a);
        cycles
    }

    fn tsx(&mut self, cycles: u8) -> u8 {
        self.x = self.sp;
        self.set_zn(self.x);
        cycles
    }

    fn txs(&mut self, cycles: u8) -> u8 {
        self.sp = self.x;
        cycles
    }

    fn pha(&mut self, mem: &mut dyn Memory, cycles: u8) -> u8 {
        self.stack_push(mem, self.a);
        cycles
    }

    fn php(&mut self, mem: &mut dyn Memory, cycles: u8) -> u8 {
        self.stack_push(mem, self.p);
        cycles
    }

    fn pla(&mut self, mem: &mut dyn Memory, cycles: u8) -> u8 {
        self.a = self.stack_pop(mem);
        cycles
    }

    fn plp(&mut self, mem: &mut dyn Memory, cycles: u8) -> u8 {
        self.p = self.stack_pop(mem);
        cycles
    }

    fn and(&mut self, mem: &mut dyn Memory, mode: AddrMode, cycles: u8) -> u8 {
        let f = self.fetch(mem, mode);
        self.a = self.a & f.value;
        self.set_zn(self.a);
        cycles
    }

    fn eor(&mut self, mem: &mut dyn Memory, mode: AddrMode, cycles: u8) -> u8 {
        let f = self.fetch(mem, mode);
        self.a = self.a ^ f.value;
        self.set_zn(self.a);
        cycles
    }

    fn ora(&mut self, mem: &mut dyn Memory, mode: AddrMode, cycles: u8) -> u8 {
        let f = self.fetch(mem, mode);
        self.a = self.a | f.value;
        self.set_zn(self.a);
        cycles
    }

    fn bit(&mut self, mem: &mut dyn Memory, mode: AddrMode, cycles: u8) -> u8 {
        let f = self.fetch(mem, mode);
        let result = f.value & self.a;
        self.set_flag(FL_ZERO, result == 0);
        self.set_flag(FL_OVERFLOW, result & (1 << 6) != 0);
        self.set_flag(FL_NEGATIVE, result & (1 << 7) != 0);
        cycles
    }

    fn asl(&mut self, mem: &mut dyn Memory, mode: AddrMode, cycles: u8) -> u8 {
        let f = self.fetch(mem, mode.clone());

        let result = f.value << 1;
        self.set_flag(FL_NEGATIVE, result & (1 << 7) != 0);
        self.set_flag(FL_CARRY, f.value & (1 << 7) != 0);
        self.set_flag(FL_ZERO, result == 0);

        match mode {
            AddrMode::Acc => self.a = result,
            _ => mem.write(f.addr, result),
        }

        cycles
    }

    fn lsr(&mut self, mem: &mut dyn Memory, mode: AddrMode, cycles: u8) -> u8 {
        let f = self.fetch(mem, mode.clone());

        let result = f.value >> 1;
        self.set_flag(FL_NEGATIVE, result & (1 << 7) != 0);
        self.set_flag(FL_CARRY, f.value & 1 != 0);
        self.set_flag(FL_ZERO, result == 0);

        match mode {
            AddrMode::Acc => self.a = result,
            _ => mem.write(f.addr, result),
        }

        cycles
    }

    fn rol(&mut self, mem: &mut dyn Memory, mode: AddrMode, cycles: u8) -> u8 {
        let f = self.fetch(mem, mode.clone());

        let result = (f.value << 1) | self.read_flag(FL_CARRY) as Byte;
        self.set_flag(FL_NEGATIVE, result & (1 << 7) != 0);
        self.set_flag(FL_CARRY, f.value & (1 << 7) != 0);
        self.set_flag(FL_ZERO, result == 0);

        match mode {
            AddrMode::Acc => self.a = result,
            _ => mem.write(f.addr, result),
        }

        cycles
    }

    fn ror(&mut self, mem: &mut dyn Memory, mode: AddrMode, cycles: u8) -> u8 {
        let f = self.fetch(mem, mode.clone());

        let result = (f.value >> 1) | (self.read_flag(FL_CARRY) as Byte) << 7;
        self.set_flag(FL_NEGATIVE, result & (1 << 7) != 0);
        self.set_flag(FL_CARRY, f.value & 1 != 0);
        self.set_flag(FL_ZERO, result == 0);

        match mode {
            AddrMode::Acc => self.a = result,
            _ => mem.write(f.addr, result),
        }

        cycles
    }

    pub fn reset(&mut self, reset_vec: Word) {
        self.pc = reset_vec;
        self.sp = 0xFF;
        self.p = 0;

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
    println!("P:  0b{:08b}", cpu.p);
    println!("PC: 0x{:04X}", cpu.pc);
    println!("SP: 0x{:02X}", cpu.sp);
    println!("A:  0x{:02X}", cpu.a);
    println!("X:  0x{:02X}", cpu.x);
    println!("Y:  0x{:02X}", cpu.y);
}

#[cfg(test)]
#[path = "cpu_test.rs"]
mod cpu_test;
