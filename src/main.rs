type Byte = u8;
type Word = u16;
type Flag = u8;

const MEM_SIZE: usize = 1024 * 10;

pub struct Memory {
    ram: [Byte; MEM_SIZE],
}

impl Memory {
    pub fn new() -> Memory {
        Memory { ram: [0; MEM_SIZE] }
    }

    pub fn read_byte(&self, addr: Word) -> Byte {
        self.ram[addr as usize]
    }

    pub fn write_byte(&mut self, addr: Word, value: Byte) {
        self.ram[addr as usize] = value;
    }

    pub fn read_word(&self, addr: Word) -> Word {
        let lo = self.read_byte(addr) as Word;
        let hi = self.read_byte(addr + 1) as Word;
        (hi << 8) | lo
    }

    pub fn write_word(&mut self, addr: Word, data: Word) {
        let lo = data as Byte;
        let hi = (data >> 8) as Byte;
        self.write_byte(addr, lo);
        self.write_byte(addr + 1, hi)
    }
}


const FLAG_CARRY: Flag = (1 << 0);
const FLAG_ZERO: Flag = (1 << 1);
const FLAG_NO_INTERRUPT: Flag = (1 << 2);
const FLAG_DECIMAL: Flag = (1 << 3);
const FLAG_BREAK: Flag = (1 << 4);
const FLAG_OVERFLOW: Flag = (1 << 6);
const FLAG_NEGATIVE: Flag = (1 << 7);

pub struct CPU {
    flags: Byte, // Status flags
    sp: Byte,    // Stack pointer
    pc: Word,    // Program counter

    a: Byte, // Accumulator
    x: Byte, // X register
    y: Byte, // Y register
}

const OP_LDA_IMM: Byte = 0xA9;
const OP_LDA_ZP: Byte = 0xA5;
const OP_LDA_ZPX: Byte = 0xB5;

impl CPU {
    fn new() -> CPU {
        CPU {
            flags: 0,
            sp: 0,
            pc: 0,
            a: 0,
            x: 0,
            y: 0,
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

    fn fetch(&mut self, mem: &Memory) -> Byte {
        let data = mem.read_byte(self.pc);
        self.pc += 1;
        data
    }

    fn run_opcode(&mut self, opcode: Byte, mem: &mut Memory) {
        match opcode {
            OP_LDA_IMM => self.lda_imm(mem),
            OP_LDA_ZP => self.lda_zp(mem),
            OP_LDA_ZPX => self.lda_zpx(mem),
            _ => panic!("unknown opcode: {:02X}", opcode),
        }
    }

    fn addr_zp(&mut self, mem: &Memory) -> Word {
        let zp_addr = self.fetch(mem);
        zp_addr as Word
    }

    fn lda_finalize(&mut self) {
        self.set_flag(FLAG_ZERO, self.a == 0x00);
        self.set_flag(FLAG_NEGATIVE, self.a & (1 << 7) > 0); // set if bit 7 of A is set
    }

    fn lda_imm(&mut self, mem: &Memory) {
        self.a = self.fetch(mem);
        self.lda_finalize();
    }

    fn lda_zp(&mut self, mem: &Memory) {
        let addr = self.addr_zp(mem);
        self.a = mem.read_byte(addr);
        self.pc += 1;
        self.lda_finalize();
    }

    fn lda_zpx(&mut self, mem: &Memory) {
        let addr = self.fetch(mem) + self.x;
        self.a = mem.read_byte(addr as Word);
        self.pc += 1;
        self.lda_finalize();
    }

    pub fn start_loop(&mut self, mem: &mut Memory) {
        let mut opcode: Byte;

        loop {
            opcode = self.fetch(mem);
            self.run_opcode(opcode, mem);
            self.print_state();
        }
    }

    fn print_state(&self) {
        println!("--- step ---");
        println!("FL: 0b{:08b}", self.flags);
        println!("PC: 0x{:04X}", self.pc);
        println!("SP: 0x{:02X}", self.sp);
        println!("A:  0x{:02X}", self.a);
        println!("X:  0x{:02X}", self.x);
        println!("Y:  0x{:02X}", self.y);
    }
}

fn main() {
    let mut cpu = CPU::new();
    let mut mem = Memory::new();

    // LDA $AA
    mem.write_byte(0x00, OP_LDA_IMM);
    mem.write_byte(0x01, 0xAA);

    mem.write_byte(0x0011, 0xAB);

    // LDA #11
    mem.write_byte(0x02, OP_LDA_ZP);
    mem.write_byte(0x03, 0x11);

    cpu.start_loop(&mut mem);
}
