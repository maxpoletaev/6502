use super::*;
use crate::mem::{Memory, Ram};

fn setup() -> (CPU, Ram) {
    let mut cpu = CPU::new();
    cpu.reset(0xFF00);

    let mem = Ram::new();
    (cpu, mem)
}

struct OpcodeTest {
    cpu: CPU,
    mem: Ram,
}

impl OpcodeTest {
    const RESET_VEC: Word = 0xFF00;

    fn new() -> Self {
        let mut cpu = CPU::new();
        cpu.reset(Self::RESET_VEC);
        let mem = Ram::new();
        Self { cpu, mem }
    }

    fn exec(&mut self, opcode: Byte, operand: Word) {
        self.mem.write(Self::RESET_VEC, opcode);

        let lsb = operand as Byte;
        let msb = (operand >> 8) as Byte;

        self.mem.write(Self::RESET_VEC + 1, lsb);
        self.mem.write(Self::RESET_VEC + 2, msb);
        self.cpu.tick(&mut self.mem);
    }

    fn assert_cycles(&self, n: u8) {
        assert_eq!(n, self.cpu.cycles, "cycles={}, want {}", self.cpu.cycles, n);
    }

    fn assert_a(&self, v: Byte) {
        assert_eq!(v, self.cpu.a, "A=0x{:02X}, want 0x{:02X}", self.cpu.a, v);
    }

    fn assert_x(&self, v: Byte) {
        assert_eq!(v, self.cpu.x, "X=0x{:02X}, want 0x{:02X}", self.cpu.x, v);
    }

    fn assert_y(&self, v: Byte) {
        assert_eq!(v, self.cpu.y, "Y=0x{:02X}, want 0x{:02X}", self.cpu.y, v);
    }

    fn assert_pc(&self, v: Word) {
        assert_eq!(v, self.cpu.pc, "PC=0x{:04X}, want 0x{:04X}", self.cpu.pc, v);
    }

    fn assert_sp(&self, v: Byte) {
        assert_eq!(v, self.cpu.sp, "SP=0x{:04X}, want 0x{:04X}", self.cpu.sp, v);
    }

    fn assert_p(&self, v: Byte) {
        assert_eq!(v, self.cpu.p, "P=0b{:08b}, want 0b{:08b}", self.cpu.p, v);
    }

    fn assert_mem(&self, addr: Word, v: Byte) {
        assert_eq!(
            v,
            self.mem.read(addr),
            "mem[0x{:04X}]=0x{:02X}, want 0x{:02X}",
            addr,
            self.mem.read(addr),
            v
        );
    }

    fn assert_zn(&self, val: Byte) {
        let want_z = val == 0;
        let want_n = (val & (1 << 7)) != 0;
        let z = self.cpu.read_flag(FL_ZERO);
        let n = self.cpu.read_flag(FL_NEGATIVE);
        assert_eq!(z, want_z, "Z=0x{}, want {}", z, want_z,);
        assert_eq!(n, want_n, "N={}, want {}", n, want_n);
    }

    fn assert_flag_set(&self, fl: Flag) {
        assert_eq!(true, self.cpu.read_flag(fl), "flag {} not set", fl);
    }

    fn assert_flag_unset(&self, fl: Flag) {
        assert_eq!(false, self.cpu.read_flag(fl), "flag {} set", fl);
    }
}

macro_rules! opcode_test {
    ($name:ident, $fn:expr) => {
        #[test]
        fn $name() {
            $fn(OpcodeTest::new());
        }
    };
}

mod lda_test {
    use super::*;

    // LDA #$AA
    opcode_test!(lda_imm, |mut t: OpcodeTest| {
        t.exec(OP_LDA_IMM, 0xAA);
        t.assert_cycles(2);
        t.assert_a(0xAA);
    });

    // LDA $10
    opcode_test!(lda_zp, |mut t: OpcodeTest| {
        t.mem.write(0x0010, 0xAA);

        t.exec(OP_LDA_ZP0, 0x10);

        t.assert_cycles(3);
        t.assert_a(0xAA);
        t.assert_flag_set(FL_NEGATIVE);
        t.assert_flag_unset(FL_ZERO);
    });

    // LDA $10,X
    opcode_test!(lda_zpx, |mut t: OpcodeTest| {
        t.mem.write(0x0011, 0x0A);

        t.cpu.x = 0x01;
        t.exec(OP_LDA_ZPX, 0x10);

        t.assert_cycles(4);
        t.assert_a(0x0A);
        t.assert_flag_unset(FL_ZERO);
        t.assert_flag_unset(FL_NEGATIVE);
    });

    // LDA $3033
    opcode_test!(lda_abs, |mut t: OpcodeTest| {
        t.mem.write(0x3033, 0x11);
        t.exec(OP_LDA_ABS, 0x3033);

        t.assert_cycles(4);
        t.assert_a(0x11);
    });

    // LDA $AAA9,X
    opcode_test!(lda_abx, |mut t: OpcodeTest| {
        t.mem.write(0xAAAA, 0x11);
        t.cpu.x = 0x01;

        t.exec(OP_LDA_ABX, 0xAAA9);
        t.assert_cycles(4);
        t.assert_a(0x11);
    });

    // LDA $AAA9,Y
    opcode_test!(lda_aby, |mut t: OpcodeTest| {
        t.mem.write(0xAAAA, 0x11);
        t.cpu.y = 0x01;

        t.exec(OP_LDA_ABY, 0xAAA9);
        t.assert_cycles(4);
        t.assert_a(0x11);
    });

    // LDA ($A0,X)
    opcode_test!(lda_idx, |mut t: OpcodeTest| {
        // value to be loaded
        t.mem.write(0x01FF, 0x11);

        // address of the target value
        t.mem.write(0x00A2, 0xFF);
        t.mem.write(0x00A3, 0x01);

        // index
        t.cpu.x = 0x02;

        t.exec(OP_LDA_IDX, 0xA0);
        t.assert_cycles(6);
        t.assert_a(0x11);
    });

    // LDY ($A0),Y
    opcode_test!(lda_idy, |mut t: OpcodeTest| {
        // value to be loaded
        t.mem.write(0x0100, 0x11);

        // address of the target value
        t.mem.write(0x00A0, 0xFF);
        t.mem.write(0x00A1, 0x00);

        // value to be added to the address
        t.cpu.y = 0x01;

        t.exec(OP_LDA_IDY, 0xA0);
        t.assert_cycles(6);
        t.assert_a(0x11);
    });
}

mod sta_test {
    use super::*;

    #[test]
    fn sta_zp() {
        let (mut cpu, mut mem) = setup();

        // value to be stored in memory
        cpu.a = 0x11;

        // STA $02
        mem.write(0xFF00, OP_STA_ZP0);
        mem.write(0xFF01, 0x02);

        cpu.tick(&mut mem);
        assert_eq!(3, cpu.cycles);
        assert_eq!(0x11, mem.read(0x0002));
    }

    #[test]
    fn sta_zpx() {
        let (mut cpu, mut mem) = setup();

        // value to be stored
        cpu.a = 0x11;

        // address offset
        cpu.x = 0x01;

        // STA $01,X
        mem.write(0xFF00, OP_STA_ZPX);
        mem.write(0xFF01, 0x01);

        cpu.tick(&mut mem);
        assert_eq!(4, cpu.cycles);
        assert_eq!(0x11, mem.read(0x0002));
    }

    #[test]
    fn sta_abs() {
        let (mut cpu, mut mem) = setup();

        // value to be stored in memory
        cpu.a = 0x11;

        // STA $ABCD
        mem.write(0xFF00, OP_STA_ABS);
        mem.write(0xFF01, 0xCD);
        mem.write(0xFF02, 0xAB);

        cpu.tick(&mut mem);
        assert_eq!(4, cpu.cycles);
        assert_eq!(0x11, mem.read(0xABCD));
    }

    #[test]
    fn sta_abx() {
        let (mut cpu, mut mem) = setup();

        // value to be stored in memory
        cpu.a = 0x11;

        // address offset
        cpu.x = 0x01;

        // STA $ABCD
        mem.write(0xFF00, OP_STA_ABX);
        mem.write(0xFF01, 0xCC);
        mem.write(0xFF02, 0xAB);

        cpu.tick(&mut mem);
        assert_eq!(5, cpu.cycles);
        assert_eq!(0x11, mem.read(0xABCD));
    }

    #[test]
    fn sta_aby() {
        let (mut cpu, mut mem) = setup();

        // value to be stored in memory
        cpu.a = 0x11;

        // address offset
        cpu.y = 0x01;

        // STA $ABCD
        mem.write(0xFF00, OP_STA_ABY);
        mem.write(0xFF01, 0xCC);
        mem.write(0xFF02, 0xAB);

        cpu.tick(&mut mem);
        assert_eq!(5, cpu.cycles);
        assert_eq!(0x11, mem.read(0xABCD));
    }

    #[test]
    fn sta_idx() {
        let (mut cpu, mut mem) = setup();

        // value to be stored in memory
        cpu.a = 0x11;

        // pointer to target memory
        mem.write(0x00A0, 0xCD);
        mem.write(0x00A1, 0xAB);

        // STA ($A0,X)
        mem.write(0xFF00, OP_STA_IDX);
        mem.write(0xFF01, 0xA0);

        cpu.tick(&mut mem);
        assert_eq!(6, cpu.cycles);
        assert_eq!(0x11, mem.read(0xABCD));
    }

    #[test]
    fn sta_idy() {
        let (mut cpu, mut mem) = setup();

        // value to be stored in memory
        cpu.a = 0x11;

        // pointer to target memory
        mem.write(0x00A0, 0xCD);
        mem.write(0x00A1, 0xAB);

        // STA ($A0),Y
        mem.write(0xFF00, OP_STA_IDY);
        mem.write(0xFF01, 0xA0);

        cpu.tick(&mut mem);
        assert_eq!(6, cpu.cycles);
        assert_eq!(0x11, mem.read(0xABCD));
    }
}

mod ldx_test {
    use super::*;

    // LDX #$11
    opcode_test!(ldx_imm, |mut t: OpcodeTest| {
        t.cpu.x = 0x00;

        t.exec(OP_LDX_IMM, 0x11);
        t.assert_cycles(2);
        t.assert_x(0x11);
        t.assert_zn(0x11);
    });

    // LDX $AA
    opcode_test!(ldx_zp, |mut t: OpcodeTest| {
        t.mem.write(0x00AA, 0x11);

        t.exec(OP_LDX_ZP0, 0xAA);
        t.assert_cycles(3);
        t.assert_x(0x11);
        t.assert_zn(0x11);
    });

    // LDX $A9,Y
    opcode_test!(ldx_zpx, |mut t: OpcodeTest| {
        t.mem.write(0x00AA, 0x11);
        t.cpu.y = 0x01;

        t.exec(OP_LDX_ZPY, 0xA9);
        t.assert_cycles(4);
        t.assert_x(0x11);
        t.assert_zn(0x11);
    });

    // LDX $ABCD
    opcode_test!(ldx_abs, |mut t: OpcodeTest| {
        t.mem.write(0xABCD, 0x11);

        t.exec(OP_LDX_ABS, 0xABCD);
        t.assert_cycles(4);
        t.assert_x(0x11);
        t.assert_zn(0x11);
    });

    // LDX $ABCC,Y
    opcode_test!(ldx_aby, |mut t: OpcodeTest| {
        t.mem.write(0xABCD, 0x11);
        t.cpu.y = 0x01;

        t.exec(OP_LDX_ABY, 0xABCC);
        t.assert_cycles(4);
        t.assert_x(0x11);
        t.assert_zn(0x11);
    });
}

mod ldy_test {
    use super::*;

    // LDY #$11
    opcode_test!(immediate, |mut t: OpcodeTest| {
        t.cpu.y = 0x00;

        t.exec(OP_LDY_IMM, 0x11);
        t.assert_cycles(2);
        t.assert_y(0x11);
        t.assert_zn(0x11);
    });

    // LDY $AA
    opcode_test!(zero_page, |mut t: OpcodeTest| {
        t.mem.write(0x00AA, 0x11);

        t.exec(OP_LDY_ZP0, 0xAA);
        t.assert_cycles(3);
        t.assert_y(0x11);
        t.assert_zn(0x11);
    });

    // LDY $A9,X
    opcode_test!(zero_page_x, |mut t: OpcodeTest| {
        t.mem.write(0x00AA, 0x11);
        t.cpu.x = 0x01;

        t.exec(OP_LDY_ZPX, 0xA9);
        t.assert_cycles(4);
        t.assert_y(0x11);
        t.assert_zn(0x11);
    });

    // LDY $ABCD
    opcode_test!(absolute, |mut t: OpcodeTest| {
        t.mem.write(0xABCD, 0x11);

        t.exec(OP_LDY_ABS, 0xABCD);
        t.assert_cycles(4);
        t.assert_y(0x11);
        t.assert_zn(0x11);
    });

    // LDY $ABCC,X
    opcode_test!(absolute_x, |mut t: OpcodeTest| {
        t.mem.write(0xABCD, 0x11);
        t.cpu.x = 0x01;

        t.exec(OP_LDY_ABX, 0xABCC);
        t.assert_cycles(4);
        t.assert_y(0x11);
        t.assert_zn(0x11);
    });
}

mod stx_test {
    use super::*;

    // STX $AA
    opcode_test!(stx_zp, |mut t: OpcodeTest| {
        t.cpu.x = 0x11;

        t.exec(OP_STX_ZP0, 0xAA);
        t.assert_cycles(3);
        t.assert_mem(0xAA, 0x11);
    });

    // STX $A9,Y
    opcode_test!(stx_zpy, |mut t: OpcodeTest| {
        t.cpu.x = 0x11;
        t.cpu.y = 0x01;

        t.exec(OP_STX_ZPY, 0xA9);
        t.assert_cycles(4);
        t.assert_mem(0xAA, 0x11);
    });

    // STX $ABCD
    opcode_test!(stx_abs, |mut t: OpcodeTest| {
        t.cpu.x = 0x11;

        t.exec(OP_STX_ABS, 0xABCD);
        t.assert_cycles(4);
        t.assert_mem(0xABCD, 0x11);
    });
}

mod sty_test {
    use super::*;

    // STY $AA
    opcode_test!(sty_zp, |mut t: OpcodeTest| {
        t.cpu.y = 0x11;

        t.exec(OP_STY_ZP0, 0xAA);
        t.assert_cycles(3);
        t.assert_mem(0xAA, 0x11);
    });

    // STY $A9,X
    opcode_test!(sty_zpx, |mut t: OpcodeTest| {
        t.cpu.y = 0x11;
        t.cpu.x = 0x01;

        t.exec(OP_STY_ZPX, 0xA9);
        t.assert_cycles(4);
        t.assert_mem(0xAA, 0x11);
    });

    // STY $ABCD
    opcode_test!(sty_abs, |mut t: OpcodeTest| {
        t.cpu.y = 0x11;

        t.exec(OP_STY_ABS, 0xABCD);
        t.assert_cycles(4);
        t.assert_mem(0xABCD, 0x11);
    });
}

mod inc_test {
    use super::*;

    #[test]
    fn inc_zp() {
        let (mut cpu, mut mem) = setup();

        // value to be incremented
        mem.write(0x00AA, 0x01);

        // INC $AA
        mem.write(0xFF00, OP_INC_ZP0);
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
}

mod inx_test {
    use super::*;

    #[test]
    fn inx_imp() {
        let (mut cpu, mut mem) = setup();

        // INC X
        mem.write(0xFF00, OP_INX_IMP);
        cpu.x = 0x01;

        cpu.tick(&mut mem);
        assert_eq!(2, cpu.cycles);
        assert_eq!(0x02, cpu.x);
    }
}

mod iny_test {
    use super::*;

    #[test]
    fn iny_imp() {
        let (mut cpu, mut mem) = setup();

        // INC Y
        mem.write(0xFF00, OP_INY_IMP);
        cpu.y = 0x01;

        cpu.tick(&mut mem);
        assert_eq!(2, cpu.cycles);
        assert_eq!(0x02, cpu.y);
    }
}

mod dec_test {
    use super::*;

    opcode_test!(dec_zp, |mut t: OpcodeTest| {
        t.mem.write(0x00AA, 0x05);

        t.exec(OP_DEC_ZP0, 0xAA);
        t.assert_cycles(5);
        t.assert_mem(0xAA, 0x04);
        t.assert_zn(0x04);
    });

    opcode_test!(dec_zpx, |mut t: OpcodeTest| {
        t.cpu.x = 0x01;
        t.mem.write(0x00AA, 0x05);

        t.exec(OP_DEC_ZPX, 0xA9);
        t.assert_cycles(6);
        t.assert_mem(0xAA, 0x04);
        t.assert_zn(0x04);
    });

    opcode_test!(dec_abs, |mut t: OpcodeTest| {
        t.mem.write(0xAABB, 0x05);

        t.exec(OP_DEC_ABS, 0xAABB);
        t.assert_cycles(6);
        t.assert_mem(0xAABB, 0x04);
        t.assert_zn(0x04);
    });

    opcode_test!(dec_abx, |mut t: OpcodeTest| {
        t.cpu.x = 0x01;
        t.mem.write(0xAABB, 0x05);

        t.exec(OP_DEC_ABX, 0xAABA);
        t.assert_cycles(7);
        t.assert_mem(0xAABB, 0x04);
        t.assert_zn(0x04);
    });
}

mod dex_test {
    use super::*;

    opcode_test!(dex_imp, |mut t: OpcodeTest| {
        t.cpu.x = 0x05;

        t.exec(OP_DEX_IMP, 0x00);
        t.assert_cycles(2);
        t.assert_x(0x04);
        t.assert_zn(0x04);
    });
}

mod dey_test {
    use super::*;

    opcode_test!(dex_imp, |mut t: OpcodeTest| {
        t.cpu.y = 0x05;

        t.exec(OP_DEY_IMP, 0x00);
        t.assert_cycles(2);
        t.assert_y(0x04);
        t.assert_zn(0x04);
    });
}

mod jmp_test {
    use super::*;

    #[test]
    fn jmp_abs() {
        let (mut cpu, mut mem) = setup();

        // JMP $AA10
        mem.write(0xFF00, OP_JMP_ABS);
        mem.write(0xFF01, 0x10);
        mem.write(0xFF02, 0xAA);

        cpu.tick(&mut mem);
        assert_eq!(3, cpu.cycles);
        assert_eq!(0xAA10, cpu.pc);
    }

    #[test]
    fn jmp_ind() {
        let (mut cpu, mut mem) = setup();

        // target value to be loaded into PC
        mem.write(0xAABB, 0xCD);
        mem.write(0xAABC, 0xAB);

        // JMP ($AABB)
        mem.write(0xFF00, OP_JMP_IND);
        mem.write(0xFF01, 0xBB);
        mem.write(0xFF02, 0xAA);

        cpu.tick(&mut mem);
        assert_eq!(5, cpu.cycles);
        assert_eq!(0xABCD, cpu.pc);
    }

    #[test]
    fn jmp_ind_bound() {
        let (mut cpu, mut mem) = setup();

        // target value to be loaded into PC
        mem.write(0xA0FF, 0xCD);
        mem.write(0xA000, 0xAB);

        // JMP ($A0FF)
        mem.write(0xFF00, OP_JMP_IND);
        mem.write(0xFF01, 0xFF);
        mem.write(0xFF02, 0xA0);

        cpu.tick(&mut mem);
        assert_eq!(5, cpu.cycles);
        assert_eq!(0xABCD, cpu.pc);
    }
}

mod jsr_test {
    use super::*;

    // JSR $AABB
    opcode_test!(jsr_abs, |mut t: OpcodeTest| {
        t.exec(OP_JSR_ABS, 0xAABB);

        t.assert_pc(0xAABB);
        t.assert_sp(0xFD);
        t.assert_mem(0x01FF, 0x03);
        t.assert_mem(0x01FE, 0xFF);
    });
}

mod rts_test {
    use super::*;

    opcode_test!(rts_imp, |mut t: OpcodeTest| {
        t.mem.write(0x01FF, 0x03);
        t.mem.write(0x01FE, 0xFF);
        t.cpu.sp = 0xFD;

        t.exec(OP_RTS_IMP, 0);
        t.assert_sp(0xFF);
        t.assert_pc(0xFF03);
    });
}

mod adc_test {
    use super::*;

    #[test]
    fn adc_imm() {
        let (mut cpu, mut mem) = setup();

        // ADC #$02
        mem.write(0xFF00, OP_ADC_IMM);
        mem.write(0xFF01, 0x02);
        cpu.a = 0x01;

        cpu.tick(&mut mem);
        assert_eq!(2, cpu.cycles);
        assert_eq!(0x03, cpu.a);
        assert!(!cpu.read_flag(FL_CARRY));
        assert!(!cpu.read_flag(FL_NEGATIVE));
        assert!(!cpu.read_flag(FL_OVERFLOW));
    }

    #[test]
    fn adc_carry() {
        let (mut cpu, mut mem) = setup();

        // ADC #$02
        mem.write(0xFF00, OP_ADC_IMM);
        mem.write(0xFF01, 0x02);
        cpu.a = 0xFF;

        cpu.tick(&mut mem);
        assert_eq!(0x01, cpu.a);
        assert!(cpu.read_flag(FL_CARRY));
        assert!(!cpu.read_flag(FL_OVERFLOW));
    }

    #[test]
    fn adc_overflow() {
        let (mut cpu, mut mem) = setup();

        // ADC #$04
        mem.write(0xFF00, OP_ADC_IMM);
        mem.write(0xFF01, 0b01111111);
        cpu.a = 0b00000001;

        cpu.tick(&mut mem);
        assert_eq!(0b10000000, cpu.a);
        assert!(!cpu.read_flag(FL_CARRY));
        assert!(cpu.read_flag(FL_OVERFLOW));
        assert!(cpu.read_flag(FL_NEGATIVE));
    }

    #[test]
    fn adc_zp() {
        let (mut cpu, mut mem) = setup();

        // value to be added to A
        mem.write(0x0011, 0x02);

        // ADC $11
        mem.write(0xFF00, OP_ADC_ZP0);
        mem.write(0xFF01, 0x11);
        cpu.a = 0x02;

        cpu.tick(&mut mem);
        assert_eq!(3, cpu.cycles);
        assert_eq!(0x04, cpu.a);
        assert!(!cpu.read_flag(FL_CARRY));
        assert!(!cpu.read_flag(FL_OVERFLOW));
    }

    #[test]
    fn adc_zpx() {
        let (mut cpu, mut mem) = setup();

        cpu.x = 0x01;
        cpu.a = 0x02;

        // value to be added to A
        mem.write(0x0011, 0x02);

        // ADC $10,X
        mem.write(0xFF00, OP_ADC_ZPX);
        mem.write(0xFF01, 0x10);

        cpu.tick(&mut mem);
        assert_eq!(4, cpu.cycles);
        assert_eq!(0x04, cpu.a);
    }

    #[test]
    fn adc_abs() {
        let (mut cpu, mut mem) = setup();

        cpu.a = 0x02;

        // value to be added to A
        mem.write(0xAA11, 0x02);

        // ADC $AA11
        mem.write(0xFF00, OP_ADC_ABS);
        mem.write(0xFF01, 0x11);
        mem.write(0xFF02, 0xAA);

        cpu.tick(&mut mem);
        assert_eq!(4, cpu.cycles);
        assert_eq!(0x04, cpu.a);
    }

    #[test]
    fn adc_abx() {
        let (mut cpu, mut mem) = setup();

        cpu.a = 0x02;
        cpu.x = 0x01;

        // value to be added to A
        mem.write(0xAA11, 0x02);

        // ADC $AA10,X
        mem.write(0xFF00, OP_ADC_ABX);
        mem.write(0xFF01, 0x10);
        mem.write(0xFF02, 0xAA);

        cpu.tick(&mut mem);
        assert_eq!(4, cpu.cycles);
        assert_eq!(0x04, cpu.a);
    }

    #[test]
    fn adc_aby() {
        let (mut cpu, mut mem) = setup();

        cpu.a = 0x02;
        cpu.y = 0x01;

        // value to be added to A
        mem.write(0xAA11, 0x02);

        // ADC $AA10,X
        mem.write(0xFF00, OP_ADC_ABY);
        mem.write(0xFF01, 0x10);
        mem.write(0xFF02, 0xAA);

        cpu.tick(&mut mem);
        assert_eq!(4, cpu.cycles);
        assert_eq!(0x04, cpu.a);
    }

    #[test]
    fn adc_idx() {
        let (mut cpu, mut mem) = setup();

        cpu.a = 0x02;

        // value to be added to A
        mem.write(0xAA11, 0x02);

        // pointer to value memory
        mem.write(0x0004, 0x11);
        mem.write(0x0005, 0xAA);

        // index
        cpu.x = 0x02;

        // ADC ($AA10,X)
        mem.write(0xFF00, OP_ADC_IDX);
        mem.write(0xFF01, 0x02);

        cpu.tick(&mut mem);
        assert_eq!(6, cpu.cycles);
        assert_eq!(0x04, cpu.a);
    }

    #[test]
    fn adc_idy() {
        let (mut cpu, mut mem) = setup();

        cpu.a = 0x02;

        // value to be added to A
        mem.write(0xAA11, 0x02);

        // target address
        mem.write(0x0000, 0x10);
        mem.write(0x0001, 0xAA);

        // value to be added to the address
        cpu.y = 0x01;

        // ADC ($00),Y
        mem.write(0xFF00, OP_ADC_IDY);
        mem.write(0xFF01, 0x00);

        cpu.tick(&mut mem);
        assert_eq!(5, cpu.cycles);
        assert_eq!(0x04, cpu.a);
    }
}

mod sbc_test {
    use super::*;

    opcode_test!(sbc_imm, |mut t: OpcodeTest| {
        t.cpu.a = 0x02;
        t.exec(OP_SBC_IMM, 0x01);
        t.assert_cycles(2);
        t.assert_a(0x01);
        t.assert_zn(t.cpu.a);
        t.assert_flag_set(FL_CARRY);
        t.assert_flag_unset(FL_OVERFLOW);
    });

    opcode_test!(sbc_imm_carry, |mut t: OpcodeTest| {
        t.cpu.a = 0x01;
        t.exec(OP_SBC_IMM, 0x03);
        t.assert_cycles(2);
        t.assert_a(0xFE);
        t.assert_zn(t.cpu.a);
        t.assert_flag_unset(FL_CARRY);
        t.assert_flag_set(FL_OVERFLOW);
    });
}

mod clc_test {
    use super::*;

    opcode_test!(clc_imp, |mut t: OpcodeTest| {
        t.cpu.set_flag(FL_CARRY, true);
        t.exec(OP_CLC_IMP, 0);
        t.assert_cycles(2);
        t.assert_flag_unset(FL_CARRY);
    });
}

mod cli_test {
    use super::*;

    opcode_test!(cli_imp, |mut t: OpcodeTest| {
        t.cpu.set_flag(FL_NO_INTERRUPT, true);
        t.exec(OP_CLI_IMP, 0);
        t.assert_cycles(2);
        t.assert_flag_unset(FL_NO_INTERRUPT);
    });
}

mod clv_test {
    use super::*;

    opcode_test!(clv_imp, |mut t: OpcodeTest| {
        t.cpu.set_flag(FL_OVERFLOW, true);
        t.exec(OP_CLV_IMP, 0);
        t.assert_cycles(2);
        t.assert_flag_unset(FL_OVERFLOW);
    });
}

mod sec_test {
    use super::*;

    opcode_test!(sec_imp, |mut t: OpcodeTest| {
        t.cpu.set_flag(FL_CARRY, false);
        t.exec(OP_SEC_IMP, 0);
        t.assert_cycles(2);
        t.assert_flag_set(FL_CARRY);
    });
}

mod sei_test {
    use super::*;

    opcode_test!(sei_imp, |mut t: OpcodeTest| {
        t.cpu.set_flag(FL_NO_INTERRUPT, false);
        t.exec(OP_SEI_IMP, 0);
        t.assert_cycles(2);
        t.assert_flag_set(FL_NO_INTERRUPT);
    });
}

mod bcc_test {
    use super::*;

    opcode_test!(false_, |mut t: OpcodeTest| {
        t.cpu.set_flag(FL_CARRY, true);

        t.exec(OP_BCC_REL, 4);
        t.assert_cycles(2);
        t.assert_pc(0xFF02);
    });

    opcode_test!(true_positive, |mut t: OpcodeTest| {
        t.cpu.set_flag(FL_CARRY, false);

        t.exec(OP_BCC_REL, 4);
        t.assert_cycles(4);
        t.assert_pc(0xFF06);
    });

    opcode_test!(true_negative, |mut t: OpcodeTest| {
        t.cpu.set_flag(FL_CARRY, false);

        t.exec(OP_BCC_REL, -4_i8 as Word);
        t.assert_cycles(6);
        t.assert_pc(0xFEFE);
    });
}

mod bcs_test {
    use super::*;

    opcode_test!(false_, |mut t: OpcodeTest| {
        t.cpu.set_flag(FL_CARRY, false);

        t.exec(OP_BCS_REL, 4);
        t.assert_cycles(2);
        t.assert_pc(0xFF02);
    });

    opcode_test!(true_positive, |mut t: OpcodeTest| {
        t.cpu.set_flag(FL_CARRY, true);

        t.exec(OP_BCS_REL, 4);
        t.assert_cycles(4);
        t.assert_pc(0xFF06);
    });

    opcode_test!(true_negative, |mut t: OpcodeTest| {
        t.cpu.set_flag(FL_CARRY, true);

        t.exec(OP_BCS_REL, -4_i8 as Word);
        t.assert_cycles(6);
        t.assert_pc(0xFEFE);
    });
}

mod beq_test {
    use super::*;

    opcode_test!(false_, |mut t: OpcodeTest| {
        t.cpu.set_flag(FL_ZERO, false);

        t.exec(OP_BEQ_REL, 4);
        t.assert_cycles(2);
        t.assert_pc(0xFF02);
    });

    opcode_test!(true_positive, |mut t: OpcodeTest| {
        t.cpu.set_flag(FL_ZERO, true);

        t.exec(OP_BEQ_REL, 4);
        t.assert_cycles(4);
        t.assert_pc(0xFF06);
    });

    opcode_test!(true_negative, |mut t: OpcodeTest| {
        t.cpu.set_flag(FL_ZERO, true);

        t.exec(OP_BEQ_REL, -4_i8 as Word);
        t.assert_cycles(6);
        t.assert_pc(0xFEFE);
    });
}

mod bne_test {
    use super::*;

    opcode_test!(false_, |mut t: OpcodeTest| {
        t.cpu.set_flag(FL_ZERO, true);

        t.exec(OP_BNE_REL, 4);
        t.assert_cycles(2);
        t.assert_pc(0xFF02);
    });

    opcode_test!(true_positive, |mut t: OpcodeTest| {
        t.cpu.set_flag(FL_ZERO, false);

        t.exec(OP_BNE_REL, 4);
        t.assert_cycles(4);
        t.assert_pc(0xFF06);
    });

    opcode_test!(true_negative, |mut t: OpcodeTest| {
        t.cpu.set_flag(FL_ZERO, false);

        t.exec(OP_BNE_REL, -4_i8 as Word);
        t.assert_cycles(6);
        t.assert_pc(0xFEFE);
    });
}

mod bmi_test {
    use super::*;

    opcode_test!(false_, |mut t: OpcodeTest| {
        t.cpu.set_flag(FL_NEGATIVE, false);

        t.exec(OP_BMI_REL, 4);
        t.assert_cycles(2);
        t.assert_pc(0xFF02);
    });

    opcode_test!(true_positive, |mut t: OpcodeTest| {
        t.cpu.set_flag(FL_NEGATIVE, true);

        t.exec(OP_BMI_REL, 4);
        t.assert_cycles(4);
        t.assert_pc(0xFF06);
    });

    opcode_test!(true_negative, |mut t: OpcodeTest| {
        t.cpu.set_flag(FL_NEGATIVE, true);

        t.exec(OP_BMI_REL, -4_i8 as Word);
        t.assert_cycles(6);
        t.assert_pc(0xFEFE);
    });
}

mod bpl_test {
    use super::*;

    opcode_test!(false_, |mut t: OpcodeTest| {
        t.cpu.set_flag(FL_NEGATIVE, true);

        t.exec(OP_BPL_REL, 4);
        t.assert_cycles(2);
        t.assert_pc(0xFF02);
    });

    opcode_test!(true_positive, |mut t: OpcodeTest| {
        t.cpu.set_flag(FL_NEGATIVE, false);

        t.exec(OP_BPL_REL, 4);
        t.assert_cycles(4);
        t.assert_pc(0xFF06);
    });

    opcode_test!(true_negative, |mut t: OpcodeTest| {
        t.cpu.set_flag(FL_NEGATIVE, false);

        t.exec(OP_BPL_REL, -4_i8 as Word);
        t.assert_cycles(6);
        t.assert_pc(0xFEFE);
    });
}

mod bvc_test {
    use super::*;

    opcode_test!(false_, |mut t: OpcodeTest| {
        t.cpu.set_flag(FL_OVERFLOW, true);

        t.exec(OP_BVC_REL, 4);
        t.assert_cycles(2);
        t.assert_pc(0xFF02);
    });

    opcode_test!(true_positive, |mut t: OpcodeTest| {
        t.cpu.set_flag(FL_OVERFLOW, false);

        t.exec(OP_BVC_REL, 4);
        t.assert_cycles(4);
        t.assert_pc(0xFF06);
    });

    opcode_test!(true_negative, |mut t: OpcodeTest| {
        t.cpu.set_flag(FL_OVERFLOW, false);

        t.exec(OP_BVC_REL, -4_i8 as Word);
        t.assert_cycles(6);
        t.assert_pc(0xFEFE);
    });
}

mod bvs_test {
    use super::*;

    opcode_test!(false_, |mut t: OpcodeTest| {
        t.cpu.set_flag(FL_OVERFLOW, false);

        t.exec(OP_BVS_REL, 4);
        t.assert_cycles(2);
        t.assert_pc(0xFF02);
    });

    opcode_test!(true_positive, |mut t: OpcodeTest| {
        t.cpu.set_flag(FL_OVERFLOW, true);

        t.exec(OP_BVS_REL, 4);
        t.assert_cycles(4);
        t.assert_pc(0xFF06);
    });

    opcode_test!(true_negative, |mut t: OpcodeTest| {
        t.cpu.set_flag(FL_OVERFLOW, true);

        t.exec(OP_BVS_REL, -4_i8 as Word);
        t.assert_cycles(6);
        t.assert_pc(0xFEFE);
    });
}

mod cmp_test {
    use super::*;

    opcode_test!(gt, |mut t: OpcodeTest| {
        t.cpu.a = 0x11;

        t.exec(OP_CMP_IMM, 0x05);
        t.assert_flag_set(FL_CARRY);
        t.assert_flag_unset(FL_ZERO);
        t.assert_flag_unset(FL_NEGATIVE);
    });

    opcode_test!(lt, |mut t: OpcodeTest| {
        t.cpu.a = 0x05;

        t.exec(OP_CMP_IMM, 0xFF);
        t.assert_flag_unset(FL_CARRY);
        t.assert_flag_unset(FL_ZERO);
        t.assert_flag_unset(FL_NEGATIVE);
    });

    opcode_test!(eq, |mut t: OpcodeTest| {
        t.cpu.a = 0xAA;

        t.exec(OP_CMP_IMM, 0xAA);
        t.assert_flag_set(FL_CARRY);
        t.assert_flag_set(FL_ZERO);
        t.assert_flag_unset(FL_NEGATIVE);
    });

    opcode_test!(negative, |mut t: OpcodeTest| {
        t.cpu.a = 0x01;

        t.exec(OP_CMP_IMM, 0x02);
        t.assert_flag_unset(FL_CARRY);
        t.assert_flag_unset(FL_ZERO);
        t.assert_flag_set(FL_NEGATIVE);
    });
}

mod cpx_test {
    use super::*;

    opcode_test!(gt, |mut t: OpcodeTest| {
        t.cpu.x = 0x11;

        t.exec(OP_CPX_IMM, 0x05);
        t.assert_flag_set(FL_CARRY);
        t.assert_flag_unset(FL_ZERO);
        t.assert_flag_unset(FL_NEGATIVE);
    });

    opcode_test!(lt, |mut t: OpcodeTest| {
        t.cpu.x = 0x05;

        t.exec(OP_CPX_IMM, 0xFF);
        t.assert_flag_unset(FL_CARRY);
        t.assert_flag_unset(FL_ZERO);
        t.assert_flag_unset(FL_NEGATIVE);
    });

    opcode_test!(eq, |mut t: OpcodeTest| {
        t.cpu.x = 0xAA;

        t.exec(OP_CPX_IMM, 0xAA);
        t.assert_flag_set(FL_CARRY);
        t.assert_flag_set(FL_ZERO);
        t.assert_flag_unset(FL_NEGATIVE);
    });

    opcode_test!(negative, |mut t: OpcodeTest| {
        t.cpu.x = 0x01;

        t.exec(OP_CPX_IMM, 0x02);
        t.assert_flag_unset(FL_CARRY);
        t.assert_flag_unset(FL_ZERO);
        t.assert_flag_set(FL_NEGATIVE);
    });
}

mod cpy_test {
    use super::*;

    opcode_test!(gt, |mut t: OpcodeTest| {
        t.cpu.y = 0x11;

        t.exec(OP_CPY_IMM, 0x05);
        t.assert_flag_set(FL_CARRY);
        t.assert_flag_unset(FL_ZERO);
        t.assert_flag_unset(FL_NEGATIVE);
    });

    opcode_test!(lt, |mut t: OpcodeTest| {
        t.cpu.y = 0x05;

        t.exec(OP_CPY_IMM, 0xFF);
        t.assert_flag_unset(FL_CARRY);
        t.assert_flag_unset(FL_ZERO);
        t.assert_flag_unset(FL_NEGATIVE);
    });

    opcode_test!(eq, |mut t: OpcodeTest| {
        t.cpu.y = 0xAA;

        t.exec(OP_CPY_IMM, 0xAA);
        t.assert_flag_set(FL_CARRY);
        t.assert_flag_set(FL_ZERO);
        t.assert_flag_unset(FL_NEGATIVE);
    });

    opcode_test!(negative, |mut t: OpcodeTest| {
        t.cpu.y = 0x01;

        t.exec(OP_CPY_IMM, 0x02);
        t.assert_flag_unset(FL_CARRY);
        t.assert_flag_unset(FL_ZERO);
        t.assert_flag_set(FL_NEGATIVE);
    });
}

mod tax_test {
    use super::*;

    opcode_test!(tax_imp, |mut t: OpcodeTest| {
        t.cpu.a = 0xFF;
        t.cpu.x = 0x00;

        t.exec(OP_TAX_IMP, 0);
        t.assert_cycles(2);
        t.assert_x(0xFF);
        t.assert_flag_unset(FL_ZERO);
        t.assert_flag_set(FL_NEGATIVE);
    });
}

mod tay_test {
    use super::*;

    opcode_test!(tay_imp, |mut t: OpcodeTest| {
        t.cpu.a = 0xFF;
        t.cpu.y = 0x00;

        t.exec(OP_TAY_IMP, 0);
        t.assert_cycles(2);
        t.assert_y(0xFF);
        t.assert_flag_unset(FL_ZERO);
        t.assert_flag_set(FL_NEGATIVE);
    });
}

mod txa_test {
    use super::*;

    opcode_test!(txa_imp, |mut t: OpcodeTest| {
        t.cpu.x = 0xFF;
        t.cpu.a = 0x00;

        t.exec(OP_TXA_IMP, 0);
        t.assert_cycles(2);
        t.assert_a(0xFF);
        t.assert_flag_unset(FL_ZERO);
        t.assert_flag_set(FL_NEGATIVE);
    });
}

mod tya_test {
    use super::*;

    opcode_test!(tya_imp, |mut t: OpcodeTest| {
        t.cpu.y = 0xFF;
        t.cpu.a = 0x00;

        t.exec(OP_TYA_IMP, 0);
        t.assert_cycles(2);
        t.assert_a(0xFF);
        t.assert_flag_unset(FL_ZERO);
        t.assert_flag_set(FL_NEGATIVE);
    });
}

mod tsx_test {
    use super::*;

    opcode_test!(tsx_imp, |mut t: OpcodeTest| {
        t.cpu.sp = 0xFF;
        t.cpu.x = 0x00;

        t.exec(OP_TSX_IMP, 0);
        t.assert_cycles(2);
        t.assert_x(0xFF);
        t.assert_flag_unset(FL_ZERO);
        t.assert_flag_set(FL_NEGATIVE);
    });
}

mod txs_test {
    use super::*;

    opcode_test!(tsx_imp, |mut t: OpcodeTest| {
        t.cpu.sp = 0x00;
        t.cpu.x = 0xAA;

        t.exec(OP_TXS_IMP, 0);
        t.assert_cycles(2);
        t.assert_sp(0xAA);

        // flags shouldn't be affected
        t.assert_flag_unset(FL_ZERO);
        t.assert_flag_unset(FL_NEGATIVE);
    });
}

mod pha_test {
    use super::*;

    opcode_test!(pha_imp, |mut t: OpcodeTest| {
        t.cpu.a = 0xAA;

        t.exec(OP_PHA_IMP, 0);
        t.assert_cycles(3);
        t.assert_sp(0xFE);
        t.assert_mem(0x01FF, 0xAA);
    });
}

mod php_test {
    use super::*;

    opcode_test!(php_imp, |mut t: OpcodeTest| {
        t.cpu.p = 0xAA;

        t.exec(OP_PHP_IMP, 0);
        t.assert_cycles(3);
        t.assert_sp(0xFE);
        t.assert_mem(0x01FF, 0xAA);
    });
}

mod pla_test {
    use super::*;

    opcode_test!(pla_imp, |mut t: OpcodeTest| {
        t.mem.write(0x01FF, 0x11);
        t.cpu.sp = 0xFE;
        t.cpu.a = 0x00;

        t.exec(OP_PLA_IMP, 0);
        t.assert_cycles(4);
        t.assert_sp(0xFF);
        t.assert_a(0x11);
    });
}

mod plp_test {
    use super::*;

    opcode_test!(plp_imp, |mut t: OpcodeTest| {
        t.mem.write(0x01FF, 0xFF);
        t.cpu.sp = 0xFE;
        t.cpu.p = 0x00;

        t.exec(OP_PLP_IMP, 0);
        t.assert_cycles(4);
        t.assert_sp(0xFF);
        t.assert_p(0xFF);
    });
}

mod and_test {
    use super::*;

    opcode_test!(and_imm, |mut t: OpcodeTest| {
        t.cpu.a = 0b0000_1111;

        t.exec(OP_AND_IMM, 0b0011_1100);
        t.assert_cycles(2);
        t.assert_a(0b0000_1100);
        t.assert_zn(0b0000_1100);
    });
}

mod eor_test {
    use super::*;

    opcode_test!(eor_imm, |mut t: OpcodeTest| {
        t.cpu.a = 0b0000_1111;

        t.exec(OP_EOR_IMM, 0b0011_1100);
        t.assert_cycles(2);
        t.assert_a(0b0011_0011);
        t.assert_zn(0b0011_0011);
    });
}

mod ora_test {
    use super::*;

    opcode_test!(ora_imm, |mut t: OpcodeTest| {
        t.cpu.a = 0b0000_1111;

        t.exec(OP_ORA_IMM, 0b0011_1100);
        t.assert_cycles(2);
        t.assert_a(0b0011_1111);
        t.assert_zn(0b0011_1111);
    });
}

mod bit_test {
    use super::*;

    opcode_test!(bit_zero, |mut t: OpcodeTest| {
        t.cpu.a = 0b0000_0000;
        t.mem.write(0x00AA, 0b0000_0000);

        t.exec(OP_BIT_ZP0, 0xAA);
        t.assert_flag_set(FL_ZERO);
        t.assert_flag_unset(FL_OVERFLOW);
        t.assert_flag_unset(FL_NEGATIVE);
    });

    opcode_test!(bit_6_is_set, |mut t: OpcodeTest| {
        t.cpu.a = 0b0100_0000;
        t.mem.write(0x00AA, 0b0100_0000);

        t.exec(OP_BIT_ZP0, 0xAA);
        t.assert_flag_unset(FL_ZERO);
        t.assert_flag_set(FL_OVERFLOW);
        t.assert_flag_unset(FL_NEGATIVE);
    });

    opcode_test!(bit_7_is_set, |mut t: OpcodeTest| {
        t.cpu.a = 0b1000_0000;
        t.mem.write(0x00AA, 0b1000_0000);

        t.exec(OP_BIT_ZP0, 0xAA);
        t.assert_flag_unset(FL_ZERO);
        t.assert_flag_unset(FL_OVERFLOW);
        t.assert_flag_set(FL_NEGATIVE);
    });
}

mod asl_test {
    use super::*;

    opcode_test!(asl_acc, |mut t: OpcodeTest| {
        t.cpu.a = 0b1000_1111;

        t.exec(OP_ASL_ACC, 0);
        t.assert_cycles(2);
        t.assert_a(0b0001_1110);
        t.assert_flag_set(FL_CARRY);
    });

    opcode_test!(asl_abs, |mut t: OpcodeTest| {
        t.mem.write(0xABCD, 0b1000_1111);

        t.exec(OP_ASL_ABS, 0xABCD);
        t.assert_cycles(6);
        t.assert_mem(0xABCD, 0b0001_1110);
        t.assert_flag_set(FL_CARRY);
    });
}

mod lsr_test {
    use super::*;

    opcode_test!(lsr_acc, |mut t: OpcodeTest| {
        t.cpu.a = 0b1000_1111;

        t.exec(OP_LSR_ACC, 0);
        t.assert_cycles(2);
        t.assert_a(0b0100_0111);
        t.assert_flag_set(FL_CARRY);
    });

    opcode_test!(lsr_abs, |mut t: OpcodeTest| {
        t.mem.write(0xABCD, 0b1000_1111);

        t.exec(OP_LSR_ABS, 0xABCD);
        t.assert_cycles(6);
        t.assert_mem(0xABCD, 0b0100_0111);
        t.assert_flag_set(FL_CARRY);
    });
}

mod rol_test {
    use super::*;

    opcode_test!(rol_acc, |mut t: OpcodeTest| {
        t.cpu.a = 0b0000_0001;
        t.cpu.set_flag(FL_CARRY, true);

        t.exec(OP_ROL_ACC, 0);
        t.assert_cycles(2);
        t.assert_a(0b0000_0011);
        t.assert_flag_unset(FL_CARRY);
    });

    opcode_test!(rol_abs, |mut t: OpcodeTest| {
        t.mem.write(0xABCD, 0b0000_0001);
        t.cpu.set_flag(FL_CARRY, true);

        t.exec(OP_ROL_ABS, 0xABCD);
        t.assert_cycles(6);
        t.assert_mem(0xABCD, 0b0000_0011);
        t.assert_flag_unset(FL_CARRY);
    });
}

mod ror_test {
    use super::*;

    opcode_test!(ror_acc, |mut t: OpcodeTest| {
        t.cpu.a = 0b1000_0000;
        t.cpu.set_flag(FL_CARRY, true);

        t.exec(OP_ROR_ACC, 0);
        t.assert_cycles(2);
        t.assert_a(0b1100_0000);
        t.assert_flag_unset(FL_CARRY);
    });

    opcode_test!(ror_abs, |mut t: OpcodeTest| {
        t.mem.write(0xABCD, 0b1000_0000);
        t.cpu.set_flag(FL_CARRY, true);

        t.exec(OP_ROR_ABS, 0xABCD);
        t.assert_cycles(6);
        t.assert_mem(0xABCD, 0b1100_0000);
        t.assert_flag_unset(FL_CARRY);
    });
}
