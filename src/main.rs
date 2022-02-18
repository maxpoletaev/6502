mod bus;
mod clock;
mod cpu;
mod mem;
mod opcodes;
mod types;

use std::cell::RefCell;
use std::env;
use std::fs::File;
use std::io;
use std::io::Read;
use std::process;
use std::rc::Rc;

use clap::{arg, Command};

use bus::Bus;
use cpu::{print_state, CPU};
use mem::{Memory, Ram};
use types::*;

const RESET_VECTOR: Word = 0xA000;

struct Opts {
    program: String,
    debug: bool,
}

impl Opts {
    fn from_clap() -> Self {
        let args = Command::new("mos6502")
            .args(&[
                arg!(<program> "Path to the program executable"),
                arg!(-d --debug ... "Print CPU state on each tick"),
            ])
            .get_matches();

        let program = args.value_of("program").unwrap();
        let debug = args.is_present("debug");
        Self {
            program: String::from(program),
            debug,
        }
    }
}

fn load_program(f: String, mem: &mut dyn Memory) -> io::Result<()> {
    let file = File::open(f)?;

    let mut addr = 0xA000;
    for byte in file.take(2048).bytes() {
        mem.write(addr, byte.unwrap());
        addr += 1;
    }

    Ok(())
}

fn main() {
    let opts = Opts::from_clap();

    let mut mem = Ram::new();

    load_program(opts.program, &mut mem).unwrap_or_else(|err| {
        eprintln!("failed to load program: {}", err);
        process::exit(1);
    });

    let mut bus = Bus::new();
    let mem: Rc<RefCell<dyn Memory>> = Rc::new(RefCell::new(mem));
    bus.plug_in((0x0000, 0xFFFF), Rc::clone(&mem)).unwrap();

    let mut real_tick: bool;
    let mut cpu = CPU::new();
    cpu.reset(RESET_VECTOR);

    for _ in clock::Oscillator::with_mhz(1) {
        real_tick = cpu.tick(&mut bus);
        if opts.debug && real_tick {
            print_state(&cpu);
        }
    }
}
