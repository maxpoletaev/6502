mod bus;
mod clock;
mod cpu;
mod mem;
mod opcodes;
mod types;

use bus::Bus;
use cpu::{print_state, CPU};
use mem::{Memory, Ram};
use opcodes::*;
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let mut mem = Ram::new();

    // JMP $A000
    mem.write(0xFFFC, OP_JMP_ABS);
    mem.write(0xFFFD, 0x00);
    mem.write(0xFFFE, 0xA0);

    // LDA #$AA
    mem.write(0xA000, OP_LDA_IMM);
    mem.write(0xA001, 0xAA);

    // INC $1122
    mem.write(0x1122, 0x05);
    mem.write(0xA002, OP_INC_ABS);
    mem.write(0xA003, 0x22);
    mem.write(0xA004, 0x11);

    // LDA $1122
    mem.write(0xA005, OP_LDA_ABS);
    mem.write(0xA006, 0x22);
    mem.write(0xA007, 0x11);

    let mut bus = Bus::new();
    let mem: Rc<RefCell<dyn Memory>> = Rc::new(RefCell::new(mem));
    bus.plug_in((0x0000, 0xFFFF), Rc::clone(&mem)).unwrap();

    let mut real_tick: bool;
    let mut cpu = CPU::new();
    cpu.reset();

    for _ in clock::Oscillator::with_mhz(1) {
        real_tick = cpu.tick(&mut bus);
        if real_tick {
            print_state(&cpu);
        }
    }
}
