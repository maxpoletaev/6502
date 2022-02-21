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
use clock::Oscillator;
use cpu::{print_state, CPU};
use mem::{Memory, Ram};
use types::*;

const RESET_VECTOR: Word = 0x0300;
const CPU_FREQ_MHZ: f32 = 1.0;

struct Opts {
    program: String,
    debug: bool,
}

fn parse_cli_args() -> Opts {
    let args = Command::new("mos6502")
        .args(&[
            arg!(<program> "Path to the program executable"),
            arg!(-d --debug ... "Print CPU state on each tick"),
        ])
        .get_matches();

    let program = args.value_of("program").unwrap();
    let debug = args.is_present("debug");

    Opts {
        program: String::from(program),
        debug,
    }
}

fn load_program(f: String, mem: &mut dyn Memory) -> io::Result<()> {
    let file = File::open(f)?;

    let mut addr = RESET_VECTOR;
    for byte in file.take(2048).bytes() {
        mem.write(addr, byte.unwrap());
        addr += 1;
    }

    Ok(())
}

struct Stdout {
    out: Box<dyn io::Write>,
    buf: Vec<u8>,
}

impl Stdout {
    fn new(out: Box<dyn io::Write>) -> Self {
        Self {
            out,
            buf: Vec::with_capacity(255),
        }
    }
}

impl Memory for Stdout {
    fn read(&self, _addr: Word) -> Byte {
        0
    }

    fn write(&mut self, addr: Word, data: Byte) {
        self.buf.push(data);
        if addr == 0xFF {
            self.out.write(self.buf.as_slice()).unwrap();
            self.out.flush().unwrap();
            self.buf.clear();
        }
    }
}

fn main() {
    let out = Stdout::new(Box::new(io::stdout()));
    let opts = parse_cli_args();
    let mut mem = Ram::new();

    load_program(opts.program, &mut mem).unwrap_or_else(|err| {
        eprintln!("failed to load program: {}", err);
        process::exit(1);
    });

    let mut bus = Bus::new();
    let mem: Rc<RefCell<dyn Memory>> = Rc::new(RefCell::new(mem));
    let out: Rc<RefCell<dyn Memory>> = Rc::new(RefCell::new(out));
    bus.plug_in((0x0200, 0x02FF), Rc::clone(&out)).unwrap();
    bus.plug_in((0x0000, 0xFFFF), Rc::clone(&mem)).unwrap();

    let mut real_tick: bool;
    let mut cpu = CPU::new();
    cpu.reset(RESET_VECTOR);

    let mut ticks: u64 = 0;
    let clock = Oscillator::with_frequency(CPU_FREQ_MHZ);

    for _ in clock {
        real_tick = cpu.tick(&mut bus);
        if opts.debug && real_tick {
            println!("--- tick {} ---", ticks);
            print_state(&cpu);
        }
        ticks += 1;
    }
}
