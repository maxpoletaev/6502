mod bus;
mod cpu;
mod mem;
mod opcodes;
mod types;

use bus::{Bus, Device};
use cpu::CPU;
use mem::Memory;
use opcodes::*;

fn main() {
    let mut mem = Box::new(Memory::new());

    // LDA $AA
    mem.write(0x0000, OP_LDA_IMM);
    mem.write(0x0001, 0xAA);

    // INC $0011
    mem.write(0x0002, OP_INC_ABS);
    mem.write(0x0003, 0x00);
    mem.write(0x0004, 0x11);

    // LDA #11
    mem.write(0x0005, OP_LDA_ZP);
    mem.write(0x0006, 0x11);

    let mut bus = Bus::new();
    bus.plug_in((0x0000, 0x00FF), mem).unwrap();

    let mut cpu = CPU::new();
    cpu.run_for(&mut bus, 20);
}
