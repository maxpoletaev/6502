mod bus;
mod clock;
mod cpu;
mod mem;
mod opcodes;
mod types;

use std::cell::RefCell;
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

const RESET_VECTOR: Word = 0x0300;

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

    let mut addr = 0x00;
    for byte in file.take(2048).bytes() {
        mem.write(addr, byte.unwrap());
        addr += 1;
    }

    Ok(())
}

struct Stdout {
    out: Box<dyn io::Write>,
}

impl Stdout {
    fn new(out: Box<dyn io::Write>) -> Self {
        Self { out }
    }
}

impl Memory for Stdout {
    fn read(&self, _addr: Word) -> Byte {
        0
    }

    fn write(&mut self, _addr: Word, data: Byte) {
        self.out.write(&[data]).unwrap();
        self.out.flush().unwrap();
    }
}

fn main() {
    let out = Stdout::new(Box::new(io::stdout()));
    let opts = Opts::from_clap();
    let mut mem = Ram::new();

    load_program(opts.program, &mut mem).unwrap_or_else(|err| {
        eprintln!("failed to load program: {}", err);
        process::exit(1);
    });

    let mut bus = Bus::new();
    let mem: Rc<RefCell<dyn Memory>> = Rc::new(RefCell::new(mem));
    let out: Rc<RefCell<dyn Memory>> = Rc::new(RefCell::new(out));
    bus.plug_in((0x0200, 0x02FF), Rc::clone(&out)).unwrap();
    bus.plug_in((0x0300, 0xFFFF), Rc::clone(&mem)).unwrap();

    let mut real_tick: bool;
    let mut cpu = CPU::new();
    cpu.reset(RESET_VECTOR);

    let mut ticks: u64 = 0;
    for _ in clock::Oscillator::with_mhz(1) {
        real_tick = cpu.tick(&mut bus);
        if opts.debug && real_tick {
            println!("--- tick {} ---", ticks);
            print_state(&cpu);
        }
        ticks += 1;
    }
}
