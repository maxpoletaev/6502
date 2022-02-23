mod stdout;

use clap::{arg, Command};
use std::cell::RefCell;
use std::fs::File;
use std::io;
use std::io::Read;
use std::rc::Rc;

use mos6502::bus::Bus;
use mos6502::clock::Oscillator;
use mos6502::cpu::{print_state, CPU};
use mos6502::mem::{Memory, Ram};
use mos6502::types::*;
use stdout::Stdout;

const RESET_VECTOR: Word = 0x0300;
const CPU_FREQ_MHZ: f32 = 1.0;

struct Opts {
    program: String,
    debug: bool,
}

fn parse_cli_args() -> Opts {
    let args = Command::new("vm")
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

struct VirtualMachine {
    cpu: CPU,
    bus: Bus,
    clock: Oscillator,
    debug: bool,
}

impl VirtualMachine {
    fn new() -> Self {
        let mut bus = Bus::new();

        let stdout = Stdout::new(Box::new(io::stdout()));
        let out: Rc<RefCell<dyn Memory>> = Rc::new(RefCell::new(stdout));
        bus.plug_in((0x0200, 0x02FF), Rc::clone(&out)).unwrap();

        let mem: Rc<RefCell<dyn Memory>> = Rc::new(RefCell::new(Ram::new()));
        bus.plug_in((0x0000, 0xFFFF), Rc::clone(&mem)).unwrap();

        let clock = Oscillator::with_frequency(CPU_FREQ_MHZ);
        let mut cpu = CPU::new();
        cpu.reset(RESET_VECTOR);

        Self {
            cpu,
            bus,
            clock,
            debug: false,
        }
    }

    fn load_rom_from_file(&mut self, filename: String) -> io::Result<()> {
        let file = File::open(filename)?;
        let mut addr = RESET_VECTOR;
        for byte in file.bytes() {
            self.bus.write(addr, byte.unwrap());
            addr += 1;
        }
        Ok(())
    }

    fn run_loop(&mut self) {
        let mut tick_count: u64 = 0;
        let mut real_tick;

        loop {
            self.clock.tick();
            real_tick = self.cpu.tick(&mut self.bus);
            if self.debug && real_tick {
                println!("--- tick {} ---", tick_count);
                print_state(&self.cpu);
            }
            tick_count += 1;
        }
    }
}

fn main() {
    let opts = parse_cli_args();
    let mut vm = VirtualMachine::new();
    vm.debug = opts.debug;
    vm.load_rom_from_file(opts.program).unwrap();
    vm.run_loop();
}
