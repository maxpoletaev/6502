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

    fn fetch(&mut self, bus: &mut dyn Memory) -> Byte {
        let data = bus.read(self.pc);
        self.pc += 1;
        data
    }

    fn fetch_word(&mut self, bus: &mut dyn Memory) -> Word {
        let hi = bus.read(self.pc) as Word;
        self.pc += 1;

        let lo = bus.read(self.pc) as Word;
        self.pc += 1;

        (hi << 8) | lo
    }

    fn run_opcode(&mut self, opcode: Byte, bus: &mut dyn Memory) -> u8 {
        match opcode {
            OP_LDA_IMM => self.lda_imm(bus),
            OP_LDA_ZP => self.lda_zp(bus),
            OP_LDA_ZPX => self.lda_zpx(bus),
            OP_INC_ABS => self.inc_abs(bus),
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

    fn lda_imm(&mut self, bus: &mut dyn Memory) -> u8 {
        self.a = self.fetch(bus);
        self.set_zn();
        2
    }

    fn lda_zp(&mut self, bus: &mut dyn Memory) -> u8 {
        let zp_addr = self.fetch(bus) as Word;
        self.a = bus.read(zp_addr);
        self.pc += 1;
        self.set_zn();
        3
    }

    fn lda_zpx(&mut self, bus: &mut dyn Memory) -> u8 {
        let addr = self.fetch(bus) + self.x;
        self.a = bus.read(addr as Word);
        self.pc += 1;
        4
    }

    fn inc_abs(&mut self, bus: &mut dyn Memory) -> u8 {
        let addr = self.fetch_word(bus);
        let val = bus.read(addr);
        let (new_val, _) = val.overflowing_add(1);
        bus.write(addr, new_val);
        self.set_zn();
        6
    }

    pub fn tick(&mut self, bus: &mut dyn Memory) -> bool {
        if self.cycles > 0 {
            // Since we executed the opcode in one go,
            // we just do nothing for the remaining cycles.
            self.cycles -= 1;
            return false;
        }

        let opcode = self.fetch(bus);
        self.cycles = self.run_opcode(opcode, bus);
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
mod test_cpu {
    use super::*;
    use crate::mem::{Memory, Ram};

    fn setup() -> (CPU, Ram) {
        let mem = Ram::new();
        let cpu = CPU::new();
        (cpu, mem)
    }

    fn run_loop(n: u32, cpu: &mut CPU, mem: &mut Ram) {
        for _ in 0..n {
            cpu.tick(mem);
        }
    }

    #[test]
    fn lda_imm() {
        let (mut cpu, mut bus) = setup();

        // LDA $0A
        bus.write(0x0000, OP_LDA_IMM);
        bus.write(0x0001, 0x0A);

        run_loop(2, &mut cpu, &mut bus);

        assert_eq!(cpu.a, 0x0A);
        assert!(!cpu.read_flag(FLAG_ZERO));
        assert!(!cpu.read_flag(FLAG_NEGATIVE));
    }

    #[test]
    fn lda_zp() {
        let (mut cpu, mut bus) = setup();

        bus.write(0x0010, 0xAA);
        bus.write(0x0000, OP_LDA_ZP);
        bus.write(0x0001, 0x0010);

        run_loop(3, &mut cpu, &mut bus);

        assert_eq!(cpu.a, 0xAA);
        assert!(!cpu.read_flag(FLAG_ZERO));
        assert!(cpu.read_flag(FLAG_NEGATIVE));
    }

    #[test]
    fn inc_abs() {
        let (mut cpu, mut mem) = setup();

        // INC #$AAAA
        mem.write(0xAAAA, 0x00);
        mem.write(0x0000, OP_INC_ABS);
        mem.write(0x0001, 0xAA);
        mem.write(0x0002, 0xAA);

        run_loop(5, &mut cpu, &mut mem);

        let value = mem.read(0xAAAA);
        assert_eq!(0x01, value);
    }

    #[test]
    fn inc_abs_overflow() {
        let (mut cpu, mut mem) = setup();

        // INC #$AAAA
        mem.write(0xAAAA, 0xFF);
        mem.write(0x0000, OP_INC_ABS);
        mem.write(0x0001, 0xAA);
        mem.write(0x0002, 0xAA);

        run_loop(6, &mut cpu, &mut mem);

        let value = mem.read(0xAAAA);
        assert_eq!(0x00, value);
    }
}
