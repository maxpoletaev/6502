use std::collections::HashMap;
use std::ops::{Index, IndexMut};

type Byte = u8;
type Word = u16;
type Flag = u8;

pub trait Device {
    fn read(&self, addr: Word) -> Byte;
    fn write(&mut self, addr: Word, data: Byte);
}

pub struct Bus {
    addr: Word,
    devices: HashMap<(Word, Word), Box<dyn Device>>,
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            addr: 0,
            devices: HashMap::new(),
        }
    }

    pub fn set_addr(&mut self, addr: Word) {
        self.addr = addr;
    }

    pub fn read(&self) -> Byte {
        match self.find(self.addr) {
            Some(key) => {
                let device = self.devices.get(&key).unwrap();
                device.read(self.addr - key.0)
            }
            None => 0,
        }
    }

    pub fn write(&mut self, data: Byte) {
        match self.find(self.addr) {
            Some(key) => {
                let k = key;
                let device = self.devices.get_mut(&k).unwrap();
                device.write(self.addr - key.0, data);
            }
            None => (),
        }
    }

    pub fn plug_in(
        &mut self,
        mem_range: (Word, Word),
        device: Box<dyn Device>,
    ) -> Result<(), String> {
        for r in self.devices.keys() {
            if r.0 >= mem_range.1 || r.1 <= mem_range.0 {
                return Err(format!("devices overlap: {:?} and {:?}", r, mem_range));
            }
        }
        self.devices.insert(mem_range, device);
        Ok(())
    }

    pub fn find(&self, addr: Word) -> Option<(Word, Word)> {
        for key in self.devices.keys() {
            if addr >= key.0 && addr <= key.1 {
                return Some(key.clone());
            }
        }
        None
    }
}

const MEM_SIZE: usize = 64 * 1024;

pub struct Memory {
    ram: [Byte; MEM_SIZE],
}

impl Memory {
    pub fn new() -> Memory {
        Memory { ram: [0; MEM_SIZE] }
    }

    pub fn read_word(&self, addr: Word) -> Word {
        let lo = self.read(addr) as Word;
        let hi = self.read(addr + 1) as Word;
        (hi << 8) | lo
    }

    pub fn write_word(&mut self, addr: Word, data: Word) {
        let lo = data as Byte;
        let hi = (data >> 8) as Byte;
        self.write(addr, lo);
        self.write(addr + 1, hi)
    }
}

impl Index<Word> for Memory {
    type Output = Byte;
    fn index(&self, index: Word) -> &Self::Output {
        &self.ram[index as usize]
    }
}

impl IndexMut<Word> for Memory {
    fn index_mut(&mut self, index: Word) -> &mut Self::Output {
        &mut self.ram[index as usize]
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

const FLAG_CARRY: Flag = 1 << 0;
const FLAG_ZERO: Flag = 1 << 1;
const FLAG_NO_INTERRUPT: Flag = 1 << 2;
const FLAG_DECIMAL: Flag = 1 << 3;
const FLAG_BREAK: Flag = 1 << 4;
const FLAG_OVERFLOW: Flag = 1 << 6;
const FLAG_NEGATIVE: Flag = 1 << 7;

pub struct CPU<'a> {
    flags: Byte, // Status flags
    sp: Byte,    // Stack pointer
    pc: Word,    // Program counter

    a: Byte, // Accumulator
    x: Byte, // X register
    y: Byte, // Y register

    bus: &'a mut Bus,
    r_cycles: u8, // Remaining cycles for the current instruction
    n_cycle: u32, // Absolute cycle number (debug)
}

const OP_LDA_IMM: Byte = 0xA9;
const OP_LDA_ZP: Byte = 0xA5;
const OP_LDA_ZPX: Byte = 0xB5;
const OP_NOP: Byte = 0xEA;
const OP_INC_ABS: Byte = 0xEE;

impl<'a> CPU<'a> {
    fn new(bus: &'a mut Bus) -> CPU<'a> {
        CPU {
            flags: 0,
            sp: 0,
            pc: 0,
            a: 0,
            x: 0,
            y: 0,
            bus: bus,
            r_cycles: 0,
            n_cycle: 0,
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

    fn fetch(&mut self) -> Byte {
        self.bus.set_addr(self.pc);
        let data = self.bus.read();
        self.pc += 1;
        data
    }

    fn fetch_word(&mut self) -> Word {
        self.bus.set_addr(self.pc);
        let hi = self.bus.read() as Word;
        self.pc += 1;

        self.bus.set_addr(self.pc);
        let lo = self.bus.read() as Word;
        self.pc += 1;

        (hi << 8) | lo
    }

    fn run_opcode(&mut self, opcode: Byte) {
        match opcode {
            OP_LDA_IMM => self.lda_imm(),
            OP_LDA_ZP => self.lda_zp(),
            OP_LDA_ZPX => self.lda_zpx(),
            OP_NOP => self.nop(),
            OP_INC_ABS => self.inc_abs(),
            _ => panic!("unknown opcode: {:02X}", opcode),
        }
    }

    fn addr_zp(&mut self) -> Word {
        let zp_addr = self.fetch();
        zp_addr as Word
    }

    fn nop(&mut self) {
        self.r_cycles = 2;
    }

    fn set_zn(&mut self) {
        self.set_flag(FLAG_ZERO, self.a == 0x00);
        self.set_flag(FLAG_NEGATIVE, self.a & (1 << 7) > 0); // set if bit 7 of A is set
    }

    fn lda_imm(&mut self) {
        self.r_cycles = 2;
        self.a = self.fetch();
        self.set_zn();
    }

    fn lda_zp(&mut self) {
        self.r_cycles = 3;
        let addr = self.addr_zp();
        self.bus.set_addr(addr);
        self.a = self.bus.read();
        self.pc += 1;
        self.set_zn();
    }

    fn lda_zpx(&mut self) {
        self.r_cycles = 4;
        let addr = self.fetch() + self.x;
        self.bus.set_addr(addr as Word);
        self.a = self.bus.read();
        self.pc += 1;
        self.set_zn();
    }

    fn inc_abs(&mut self) {
        self.r_cycles = 6;
        let addr = self.fetch_word();
        self.bus.set_addr(addr);

        let mut val = self.bus.read();
        if val == 0xFF {
            // overflow
            val = 0x00
        } else {
            val += 1;
        }

        self.bus.write(val)
    }

    pub fn tick(&mut self) {
        if self.r_cycles == 0 {
            let opcode = self.fetch();
            self.run_opcode(opcode);
            self.print_state();
            return;
        }

        self.r_cycles -= 1;
        self.n_cycle += 1;
    }

    pub fn run_for(&mut self, mut cycles: u8) {
        while cycles > 0 {
            self.tick();
            cycles -= 1;
        }
    }

    fn print_state(&self) {
        println!("--- tick {} ---", self.n_cycle);
        println!("FL: 0b{:08b}", self.flags);
        println!("PC: 0x{:04X}", self.pc);
        println!("SP: 0x{:02X}", self.sp);
        println!("A:  0x{:02X}", self.a);
        println!("X:  0x{:02X}", self.x);
        println!("Y:  0x{:02X}", self.y);
    }
}

#[cfg(test)]
mod test_cpu {
    use super::*;

    #[test]
    fn lda_imm() {
        let mut mem = Memory::new();

        // LDA $AA
        mem.write(0x00, OP_LDA_IMM);
        mem.write(0x01, 0xAA);

        let mut bus = Bus::new();
        bus.plug_in((0x00, 0xFF), Box::new(mem)).unwrap();
        let mut cpu = CPU::new(&mut bus);
        cpu.run_for(2);

        assert_eq!(cpu.a, 0xAA);
        assert!(!cpu.read_flag(FLAG_ZERO));
        assert!(cpu.read_flag(FLAG_NEGATIVE));
    }

    #[test]
    fn lda_imm_zero() {
        let mut mem = Memory::new();

        // LDA 00
        mem.write(0x00, OP_LDA_IMM);
        mem.write(0x01, 0x00);

        let mut bus = Bus::new();
        bus.plug_in((0x00, 0xFF), Box::new(mem)).unwrap();
        let mut cpu = CPU::new(&mut bus);
        cpu.tick();

        assert_eq!(cpu.a, 0x00);
        assert!(cpu.read_flag(FLAG_ZERO));
        assert!(!cpu.read_flag(FLAG_NEGATIVE));
    }

    #[test]
    fn lda_zp() {
        let mut mem = Memory::new();

        mem.write(0x0010, 0xAA);
        mem.write(0x00, OP_LDA_ZP);
        mem.write(0x01, 0x0010);

        let mut bus = Bus::new();
        bus.plug_in((0x00, 0xFF), Box::new(mem)).unwrap();
        let mut cpu = CPU::new(&mut bus);
        cpu.tick();

        assert_eq!(cpu.a, 0xAA);
        assert!(!cpu.read_flag(FLAG_ZERO));
        assert!(cpu.read_flag(FLAG_NEGATIVE));
    }

    #[test]
    fn inc_abs() {
        let mut mem = Box::new(Memory::new());

        // INC #$AAAA
        mem.write(0xAAAA, 0x00);
        mem.write(0x0000, OP_INC_ABS);
        mem.write(0x0001, 0xAA);
        mem.write(0x0002, 0xAA);

        let mut bus = Bus::new();
        bus.plug_in((0x0000, 0xFFFF), mem).unwrap();
        let mut cpu = CPU::new(&mut bus);
        cpu.run_for(6);

        bus.set_addr(0xAAAA);
        let value = bus.read();
        assert_eq!(0x01, value);
    }

    #[test]
    fn inc_abs_overflow() {
        let mut mem = Box::new(Memory::new());

        // INC #$AAAA
        mem.write(0xAAAA, 0xFF);
        mem.write(0x0000, OP_INC_ABS);
        mem.write(0x0001, 0xAA);
        mem.write(0x0002, 0xAA);

        let mut bus = Bus::new();
        bus.plug_in((0x0000, 0xFFFF), mem).unwrap();
        let mut cpu = CPU::new(&mut bus);
        cpu.run_for(6);

        bus.set_addr(0xAAAA);
        let value = bus.read();
        assert_eq!(0x00, value);
    }
}

fn main() {
    let mut mem = Box::new(Memory::new());

    // LDA $AA
    mem[0x0000] = OP_LDA_IMM;
    mem[0x0001] = 0xAA;

    // INC $0011
    mem[0x0002] = OP_INC_ABS;
    mem[0x0003] = 0x00;
    mem[0x0004] = 0x11;

    // LDA #11
    mem[0x0005] = OP_LDA_ZP;
    mem[0x0006] = 0x11;

    let mut bus = Bus::new();
    bus.plug_in((0x0000, 0x00FF), mem).unwrap();

    let mut cpu = CPU::new(&mut bus);
    cpu.run_for(20);
}
