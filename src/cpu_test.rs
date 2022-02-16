use super::*;
use crate::mem::{Memory, Ram};

fn setup() -> (CPU, Ram) {
    let mem = Ram::new();
    let cpu = CPU::new();
    (cpu, mem)
}

fn run_loop(n: u32, cpu: &mut CPU, mem: &mut Ram) {
    for _ in 0..n {
        cpu.tick(mem);
    }
}

#[warn(dead_code)]
fn run_loop_state(n: u32, cpu: &mut CPU, mem: &mut Ram) {
    for _ in 0..n {
        if cpu.tick(mem) {
            print_state(&cpu);
        }
    }
}

#[test]
fn lda_imm() {
    let (mut cpu, mut mem) = setup();

    // LDA #AA
    mem.write(0x0000, OP_LDA_IMM);
    mem.write(0x0001, 0xAA);

    run_loop(2, &mut cpu, &mut mem);

    assert_eq!(cpu.a, 0xAA);
}

#[test]
fn lda_zp() {
    let (mut cpu, mut mem) = setup();

    mem.write(0x0010, 0xAA);

    // LDA $10
    mem.write(0x0000, OP_LDA_ZP);
    mem.write(0x0001, 0x10);

    run_loop(3, &mut cpu, &mut mem);
    assert_eq!(cpu.a, 0xAA);
    assert!(!cpu.read_flag(FLAG_ZERO));
    assert!(cpu.read_flag(FLAG_NEGATIVE));
}

#[test]
fn lda_zpx() {
    let (mut cpu, mut mem) = setup();

    cpu.x = 0x01;
    mem.write(0x0011, 0xAA);

    // LDA $10,X
    mem.write(0x0000, OP_LDA_ZPX);
    mem.write(0x0001, 0x10);

    run_loop(4, &mut cpu, &mut mem);
    assert_eq!(cpu.a, 0xAA);
    assert!(!cpu.read_flag(FLAG_ZERO));
    assert!(cpu.read_flag(FLAG_NEGATIVE));
}

#[test]
fn lda_idx() {
    let (mut cpu, mut mem) = setup();

    cpu.x = 0x02;
    mem.write(0x00A2, 0x01); // first part of the target address
    mem.write(0x00A3, 0xFF); // second part of the target address
    mem.write(0x01FF, 0x11); // value to be loaded

    // LDA ($A0,X)
    mem.write(0x0000, OP_LDA_IDX);
    mem.write(0x0001, 0xA0);

    run_loop(6, &mut cpu, &mut mem);
    assert_eq!(cpu.a, 0x11);
}

#[test]
fn lda_idy() {
    let (mut cpu, mut mem) = setup();

    cpu.y = 0x01;
    mem.write(0x00A0, 0x00); // first part of the target address
    mem.write(0x00A1, 0xFF); // second part of the target address
    mem.write(0x0100, 0x11); // value to be loaded

    // LDA ($A0),Y
    mem.write(0x0000, OP_LDA_IDY);
    mem.write(0x0001, 0xA0);

    run_loop(6, &mut cpu, &mut mem);
    assert_eq!(cpu.a, 0x11);
}

#[test]
fn lda_abs() {
    let (mut cpu, mut mem) = setup();

    mem.write(0xAAAA, 0xBB);

    // LDA $AAAA
    mem.write(0x0000, OP_LDA_ABS);
    mem.write(0x0001, 0xAA);
    mem.write(0x0002, 0xAA);

    run_loop(4, &mut cpu, &mut mem);
    assert_eq!(cpu.a, 0xBB);
    assert!(!cpu.read_flag(FLAG_ZERO));
    assert!(cpu.read_flag(FLAG_NEGATIVE));
}

#[test]
fn inc_abs() {
    let (mut cpu, mut mem) = setup();

    // INC $AAAA
    mem.write(0xAAAA, 0x00);
    mem.write(0x0000, OP_INC_ABS);
    mem.write(0x0001, 0xAA);
    mem.write(0x0002, 0xAA);

    run_loop(5, &mut cpu, &mut mem);

    let value = mem.read(0xAAAA);
    assert_eq!(0x01, value);
}

#[test]
fn inc_abs_overflow() {
    let (mut cpu, mut mem) = setup();

    // INC #$AAAA
    mem.write(0xAAAA, 0xFF);
    mem.write(0x0000, OP_INC_ABS);
    mem.write(0x0001, 0xAA);
    mem.write(0x0002, 0xAA);

    run_loop(6, &mut cpu, &mut mem);

    let value = mem.read(0xAAAA);
    assert_eq!(0x00, value);
}
