use crate::opcodes::*;
use crate::types::*;
use crate::Bus;

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

    cycles: u8,        // Remaining cycles for the current instruction
    tick_counter: u32, // Absolute cycle number (debug)
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
            tick_counter: 0,
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

    fn fetch(&mut self, bus: &mut Bus) -> Byte {
        let data = bus.read_addr(self.pc);
        self.pc += 1;
        data
    }

    fn fetch_word(&mut self, bus: &mut Bus) -> Word {
        let hi = bus.read_addr(self.pc) as Word;
        self.pc += 1;

        let lo = bus.read_addr(self.pc) as Word;
        self.pc += 1;

        (hi << 8) | lo
    }

    fn run_opcode(&mut self, opcode: Byte, bus: &mut Bus) {
        match opcode {
            OP_LDA_IMM => self.lda_imm(bus),
            OP_LDA_ZP => self.lda_zp(bus),
            OP_LDA_ZPX => self.lda_zpx(bus),
            OP_INC_ABS => self.inc_abs(bus),
            OP_NOP => self.nop(),
            _ => panic!("unknown opcode: {:02X}", opcode),
        }
    }

    fn addr_zp(&mut self, bus: &mut Bus) -> Word {
        let zp_addr = self.fetch(bus);
        zp_addr as Word
    }

    fn nop(&mut self) {
        self.cycles = 2;
    }

    fn set_zn(&mut self) {
        self.set_flag(FLAG_ZERO, self.a == 0x00);
        self.set_flag(FLAG_NEGATIVE, self.a & (1 << 7) > 0); // set if bit 7 of A is set
    }

    fn lda_imm(&mut self, bus: &mut Bus) {
        self.cycles = 2;
        self.a = self.fetch(bus);
        self.set_zn();
    }

    fn lda_zp(&mut self, bus: &mut Bus) {
        self.cycles = 3;
        let addr = self.addr_zp(bus);
        self.a = bus.read_addr(addr);
        self.pc += 1;
        self.set_zn();
    }

    fn lda_zpx(&mut self, bus: &mut Bus) {
        self.cycles = 4;
        let addr = self.fetch(bus) + self.x;
        self.a = bus.read_addr(addr as Word);
        self.pc += 1;
        self.set_zn();
    }

    fn inc_abs(&mut self, bus: &mut Bus) {
        self.cycles = 6;
        let addr = self.fetch_word(bus);
        let val = bus.read_addr(addr);
        let (new_val, _) = val.overflowing_add(1);
        bus.write(new_val);
        self.set_zn();
    }

    pub fn tick(&mut self, bus: &mut Bus) -> bool {
        if self.cycles == 0 {
            let opcode = self.fetch(bus);
            self.run_opcode(opcode, bus);
            return true;
        }

        // Since we executed the opcode in one go,
        // we just do nothing for the remaining cycles.
        self.tick_counter += 1;
        self.cycles -= 1;
        false
    }
}

pub fn print_state(cpu: &CPU) {
    println!("--- tick {} ---", cpu.tick_counter);
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
    use crate::mem::Memory;

    fn setup_cpu() -> CPU {
        CPU::new()
    }

    fn setup_bus() -> Box<Bus> {
        let mut bus = Box::new(Bus::new());
        let mem = Box::new(Memory::new());
        bus.plug_in((0x0000, 0xFFFF), mem).unwrap();
        bus
    }

    #[test]
    fn lda_imm() {
        let mut bus = setup_bus();

        // LDA $AA
        bus.write_addr(0x0000, OP_LDA_IMM);
        bus.write_addr(0x0001, 0xAA);

        let mut cpu = setup_cpu();
        cpu.run_for(&mut bus, 2);

        assert_eq!(cpu.a, 0xAA);
    }

    #[test]
    fn lda_imm_zero() {
        let mut mem = setup_bus();

        // LDA 00
        mem.write_addr(0x00, OP_LDA_IMM);
        mem.write_addr(0x01, 0x00);

        let mut cpu = setup_cpu();
        cpu.run_for(&mut mem, 3);

        assert_eq!(cpu.a, 0x00);
        assert!(cpu.read_flag(FLAG_ZERO));
        assert!(!cpu.read_flag(FLAG_NEGATIVE));
    }

    // #[test]
    // fn lda_zp() {
    //     let mut mem = Memory::new();

    //     mem.write(0x0010, 0xAA);
    //     mem.write(0x00, OP_LDA_ZP);
    //     mem.write(0x01, 0x0010);

    //     let mut bus = Bus::new();
    //     bus.plug_in((0x00, 0xFF), Box::new(mem)).unwrap();
    //     let mut cpu = CPU::new(&mut bus);
    //     cpu.tick();

    //     assert_eq!(cpu.a, 0xAA);
    //     assert!(!cpu.read_flag(FLAG_ZERO));
    //     assert!(cpu.read_flag(FLAG_NEGATIVE));
    // }

    // #[test]
    // fn inc_abs() {
    //     let mut mem = Box::new(Memory::new());

    //     // INC #$AAAA
    //     mem.write(0xAAAA, 0x00);
    //     mem.write(0x0000, OP_INC_ABS);
    //     mem.write(0x0001, 0xAA);
    //     mem.write(0x0002, 0xAA);

    //     let mut bus = Bus::new();
    //     bus.plug_in((0x0000, 0xFFFF), mem).unwrap();
    //     let mut cpu = CPU::new(&mut bus);
    //     cpu.run_for(6);

    //     bus.set_addr(0xAAAA);
    //     let value = bus.read();
    //     assert_eq!(0x01, value);
    // }

    // #[test]
    // fn inc_abs_overflow() {
    //     let mut mem = Box::new(Memory::new());

    //     // INC #$AAAA
    //     mem.write(0xAAAA, 0xFF);
    //     mem.write(0x0000, OP_INC_ABS);
    //     mem.write(0x0001, 0xAA);
    //     mem.write(0x0002, 0xAA);

    //     let mut bus = Bus::new();
    //     bus.plug_in((0x0000, 0xFFFF), mem).unwrap();
    //     let mut cpu = CPU::new(&mut bus);
    //     cpu.run_for(6);

    //     bus.set_addr(0xAAAA);
    //     let value = bus.read();
    //     assert_eq!(0x00, value);
    // }
}
