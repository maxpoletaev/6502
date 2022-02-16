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
    let a = 1 << 7;
    println!("{:08b}", a);

    let mut mem = Ram::new();

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
    let mem: Rc<RefCell<dyn Memory>> = Rc::new(RefCell::new(mem));
    bus.plug_in((0x0000, 0x00FF), Rc::clone(&mem)).unwrap();

    let mut real_tick: bool;
    let mut cpu = CPU::new();

    for _ in clock::Oscillator::with_mhz(1) {
        real_tick = cpu.tick(&mut bus);
        if real_tick {
            print_state(&cpu);
        }
    }
}
