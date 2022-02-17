use super::*;
use crate::mem::{Memory, Ram};

fn setup() -> (CPU, Ram) {
    let mem = Ram::new();
    let mut cpu = CPU::new();
    cpu.reset();
    cpu.pc = 0xFF00;
    (cpu, mem)
}

#[test]
fn lda_imm() {
    let (mut cpu, mut mem) = setup();

    // LDA #AA
    mem.write(0xFF00, OP_LDA_IMM);
    mem.write(0xFF01, 0xAA);

    cpu.tick(&mut mem);

    assert_eq!(cpu.cycles, 2);
    assert_eq!(cpu.a, 0xAA);
}

#[test]
fn lda_zp() {
    let (mut cpu, mut mem) = setup();

    // data to be loaded
    mem.write(0x0010, 0xAA);

    // LDA $10
    mem.write(0xFF00, OP_LDA_ZP);
    mem.write(0xFF01, 0x10);

    cpu.tick(&mut mem);

    assert_eq!(0xAA, cpu.a);
    assert_eq!(3, cpu.cycles);
    assert!(!cpu.read_flag(FLAG_ZERO));
    assert!(cpu.read_flag(FLAG_NEGATIVE));
}

#[test]
fn lda_zpx() {
    let (mut cpu, mut mem) = setup();

    // value to be loaded
    mem.write(0x0011, 0xAA);

    // LDA $10,X
    mem.write(0xFF00, OP_LDA_ZPX);
    mem.write(0xFF01, 0x10);
    cpu.x = 0x01;

    cpu.tick(&mut mem);
    assert_eq!(4, cpu.cycles);
    assert_eq!(0xAA, cpu.a);
    assert!(!cpu.read_flag(FLAG_ZERO));
    assert!(cpu.read_flag(FLAG_NEGATIVE));
}

#[test]
fn lda_abx() {
    let (mut cpu, mut mem) = setup();

    // value to be loaded
    mem.write(0xAAAA, 0x11);

    // LDA $AAA9
    mem.write(0xFF00, OP_LDA_ABX);
    mem.write(0xFF01, 0xA9);
    mem.write(0xFF02, 0xAA);
    cpu.x = 0x0001;

    cpu.tick(&mut mem);
    assert_eq!(4, cpu.cycles);
    assert_eq!(0x11, cpu.a);
}

#[test]
fn lda_aby() {
    let (mut cpu, mut mem) = setup();

    // data to be loaded
    mem.write(0xAAAA, 0x11);

    // LDA $AAA9
    mem.write(0xFF00, OP_LDA_ABY);
    mem.write(0xFF01, 0xA9);
    mem.write(0xFF02, 0xAA);
    cpu.y = 0x0001;

    cpu.tick(&mut mem);
    assert_eq!(cpu.cycles, 4);
    assert_eq!(cpu.a, 0x11);
}

#[test]
fn lda_idx() {
    let (mut cpu, mut mem) = setup();

    cpu.x = 0x02;
    mem.write(0x00A2, 0xFF); // second part of the target address
    mem.write(0x00A3, 0x01); // first part of the target address
    mem.write(0x01FF, 0x11); // value to be loaded

    // LDA ($A0,X)
    mem.write(0xFF00, OP_LDA_IDX);
    mem.write(0xFF01, 0xA0);

    cpu.tick(&mut mem);
    assert_eq!(6, cpu.cycles);
    assert_eq!(0x11, cpu.a);
}

#[test]
fn lda_idy() {
    let (mut cpu, mut mem) = setup();

    cpu.y = 0x01;
    mem.write(0x00A0, 0xFF); // second part of the target address
    mem.write(0x00A1, 0x00); // first part of the target address
    mem.write(0x0100, 0x11); // value to be loaded

    // LDA ($A0),Y
    mem.write(0xFF00, OP_LDA_IDY);
    mem.write(0xFF01, 0xA0);

    cpu.tick(&mut mem);
    assert_eq!(6, cpu.cycles);
    assert_eq!(0x11, cpu.a);
}

#[test]
fn lda_abs() {
    let (mut cpu, mut mem) = setup();

    mem.write(0x3033, 0x11);

    // LDA $3033
    mem.write(0xFF00, OP_LDA_ABS);
    mem.write(0xFF01, 0x33);
    mem.write(0xFF02, 0x30);

    cpu.tick(&mut mem);
    assert_eq!(cpu.cycles, 4);
    assert_eq!(0x11, cpu.a);
}

#[test]
fn inc_zp() {
    let (mut cpu, mut mem) = setup();

    // value to be incremented
    mem.write(0x00AA, 0x01);

    // INC $AA
    mem.write(0xFF00, OP_INC_ZP);
    mem.write(0xFF01, 0xAA);

    cpu.tick(&mut mem);
    assert_eq!(5, cpu.cycles);
    assert_eq!(0x02, mem.read(0x00AA));
}

#[test]
fn inc_zpx() {
    let (mut cpu, mut mem) = setup();

    // value to be incremented
    mem.write(0x00AA, 0x01);

    // INC $AA
    mem.write(0xFF00, OP_INC_ZPX);
    mem.write(0xFF01, 0xA9);
    cpu.x = 0x01;

    cpu.tick(&mut mem);
    assert_eq!(6, cpu.cycles);
    assert_eq!(0x02, mem.read(0x00AA));
}

#[test]
fn inc_abs() {
    let (mut cpu, mut mem) = setup();

    // value to be incremented
    mem.write(0xAABB, 0x01);

    // INC $AAAA
    mem.write(0xFF00, OP_INC_ABS);
    mem.write(0xFF01, 0xBB);
    mem.write(0xFF02, 0xAA);

    cpu.tick(&mut mem);
    assert_eq!(6, cpu.cycles);
    assert_eq!(0x02, mem.read(0xAABB));
}

#[test]
fn inc_abx() {
    let (mut cpu, mut mem) = setup();

    // value to be incremented
    mem.write(0xAABB, 0x01);

    // INC $AAAA
    mem.write(0xFF00, OP_INC_ABX);
    mem.write(0xFF01, 0xBA);
    mem.write(0xFF02, 0xAA);
    cpu.x = 0x01;

    cpu.tick(&mut mem);
    assert_eq!(7, cpu.cycles);
    assert_eq!(0x02, mem.read(0xAABB));
}
