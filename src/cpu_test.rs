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

    #[test]
    fn ldx_imm() {
        let (mut cpu, mut mem) = setup();

        // LDX #$11
        mem.write(0xFF00, OP_LDX_IMM);
        mem.write(0xFF01, 0x11);

        cpu.tick(&mut mem);
        assert_eq!(2, cpu.cycles);
        assert_eq!(0x11, cpu.x);
    }

    #[test]
    fn ldx_zp() {
        let (mut cpu, mut mem) = setup();

        // value to be loaded
        mem.write(0x00AA, 0x11);

        // LDX $AA
        mem.write(0xFF00, OP_LDX_ZP0);
        mem.write(0xFF01, 0xAA);

        cpu.tick(&mut mem);
        assert_eq!(3, cpu.cycles);
        assert_eq!(0x11, cpu.x);
    }

    #[test]
    fn ldx_zpy() {
        let (mut cpu, mut mem) = setup();

        // value to be loaded
        mem.write(0x00AA, 0x11);

        // LDX $A9,Y
        mem.write(0xFF00, OP_LDX_ZPY);
        mem.write(0xFF01, 0xA9);
        cpu.y = 0x01;

        cpu.tick(&mut mem);
        assert_eq!(4, cpu.cycles);
        assert_eq!(0x11, cpu.x);
    }

    #[test]
    fn ldx_abs() {
        let (mut cpu, mut mem) = setup();

        // value to be loaded
        mem.write(0xABCD, 0x11);

        // LDX $ABCD
        mem.write(0xFF00, OP_LDX_ABS);
        mem.write(0xFF01, 0xCD);
        mem.write(0xFF02, 0xAB);

        cpu.tick(&mut mem);
        assert_eq!(4, cpu.cycles);
        assert_eq!(0x11, cpu.x);
    }

    #[test]
    fn ldx_aby() {
        let (mut cpu, mut mem) = setup();

        // value to be loaded
        mem.write(0xABCD, 0x11);

        // LDX $ABCD
        mem.write(0xFF00, OP_LDX_ABY);
        mem.write(0xFF01, 0xCC);
        mem.write(0xFF02, 0xAB);
        cpu.y = 0x01;

        cpu.tick(&mut mem);
        assert_eq!(4, cpu.cycles);
        assert_eq!(0x11, cpu.x);
    }
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

mod clc_test {
    use super::*;

    #[test]
    fn clc_imp() {
        let (mut cpu, mut mem) = setup();

        cpu.set_flag(FL_CARRY, true);
        assert!(cpu.read_flag(FL_CARRY));

        mem.write(0xFF00, OP_CLC_IMP);

        cpu.tick(&mut mem);
        assert_eq!(2, cpu.cycles);
        assert!(!cpu.read_flag(FL_CARRY));
    }
}

mod bcc_test {
    use super::*;

    #[test]
    fn bcc_negative() {
        let (mut cpu, mut mem) = setup();

        cpu.set_flag(FL_CARRY, true);

        // BCC $10
        mem.write(0xFF00, OP_BCC_REL);
        mem.write(0xFF01, 0x10);

        cpu.tick(&mut mem);
        assert_eq!(2, cpu.cycles);
        assert_eq!(0xFF02, cpu.pc);
    }

    #[test]
    fn bcc_positive() {
        let (mut cpu, mut mem) = setup();

        cpu.set_flag(FL_CARRY, false);

        // BCC $10
        mem.write(0xFF00, OP_BCC_REL);
        mem.write(0xFF01, 0x10);

        cpu.tick(&mut mem);
        assert_eq!(4, cpu.cycles);
        assert_eq!(0xFF12, cpu.pc);
    }

    #[test]
    fn bcc_next_page() {
        let (mut cpu, mut mem) = setup();

        cpu.set_flag(FL_CARRY, false);

        // BCC $F0
        mem.write(0xFF00, OP_BCC_REL);
        mem.write(0xFF01, 0xFF);

        cpu.tick(&mut mem);
        assert_eq!(6, cpu.cycles);
        assert_eq!(0x0001, cpu.pc);
    }
}

mod bcs_test {
    use super::*;

    #[test]
    fn bcs_negative() {
        let (mut cpu, mut mem) = setup();

        cpu.set_flag(FL_CARRY, false);

        // BCS $10
        mem.write(0xFF00, OP_BCS_REL);
        mem.write(0xFF01, 0x10);

        cpu.tick(&mut mem);
        assert_eq!(2, cpu.cycles);
        assert_eq!(0xFF02, cpu.pc);
    }

    #[test]
    fn bcs_positive() {
        let (mut cpu, mut mem) = setup();

        cpu.set_flag(FL_CARRY, true);

        // BCS $10
        mem.write(0xFF00, OP_BCS_REL);
        mem.write(0xFF01, 0x10);

        cpu.tick(&mut mem);
        assert_eq!(4, cpu.cycles);
        assert_eq!(0xFF12, cpu.pc);
    }

    #[test]
    fn bcs_next_page() {
        let (mut cpu, mut mem) = setup();

        cpu.set_flag(FL_CARRY, true);

        // BCS $F0
        mem.write(0xFF00, OP_BCS_REL);
        mem.write(0xFF01, 0xFF);

        cpu.tick(&mut mem);
        assert_eq!(6, cpu.cycles);
        assert_eq!(0x0001, cpu.pc);
    }
}

mod beq_test {
    use super::*;

    #[test]
    fn beq_negative() {
        let (mut cpu, mut mem) = setup();

        cpu.set_flag(FL_ZERO, false);

        // BCS $10
        mem.write(0xFF00, OP_BEQ_REL);
        mem.write(0xFF01, 0x10);

        cpu.tick(&mut mem);
        assert_eq!(2, cpu.cycles);
        assert_eq!(0xFF02, cpu.pc);
    }

    #[test]
    fn beq_positive() {
        let (mut cpu, mut mem) = setup();

        cpu.set_flag(FL_ZERO, true);

        // BCS $10
        mem.write(0xFF00, OP_BEQ_REL);
        mem.write(0xFF01, 0x10);

        cpu.tick(&mut mem);
        assert_eq!(4, cpu.cycles);
        assert_eq!(0xFF12, cpu.pc);
    }

    #[test]
    fn beq_next_page() {
        let (mut cpu, mut mem) = setup();

        cpu.set_flag(FL_ZERO, true);

        // BCS $F0
        mem.write(0xFF00, OP_BEQ_REL);
        mem.write(0xFF01, 0xFF);

        cpu.tick(&mut mem);
        assert_eq!(6, cpu.cycles);
        assert_eq!(0x0001, cpu.pc);
    }
}

mod bne_test {
    use super::*;

    #[test]
    fn bne_negative() {
        let (mut cpu, mut mem) = setup();

        cpu.set_flag(FL_ZERO, true);

        // BCS $10
        mem.write(0xFF00, OP_BNE_REL);
        mem.write(0xFF01, 0x10);

        cpu.tick(&mut mem);
        assert_eq!(2, cpu.cycles);
        assert_eq!(0xFF02, cpu.pc);
    }

    #[test]
    fn bne_positive() {
        let (mut cpu, mut mem) = setup();

        cpu.set_flag(FL_ZERO, false);

        // BCS $10
        mem.write(0xFF00, OP_BNE_REL);
        mem.write(0xFF01, 0x10);

        cpu.tick(&mut mem);
        assert_eq!(4, cpu.cycles);
        assert_eq!(0xFF12, cpu.pc);
    }

    #[test]
    fn bne_next_page() {
        let (mut cpu, mut mem) = setup();

        cpu.set_flag(FL_ZERO, false);

        // BCS $F0
        mem.write(0xFF00, OP_BNE_REL);
        mem.write(0xFF01, 0xFF);

        cpu.tick(&mut mem);
        assert_eq!(6, cpu.cycles);
        assert_eq!(0x0001, cpu.pc);
    }
}

mod cmp_test {
    use super::*;

    #[test]
    fn cmp_imm() {
        let (mut cpu, mut mem) = setup();

        // value to compare
        cpu.a = 0x11;

        // CMP #$05
        mem.write(0xFF00, OP_CMP_IMM);
        mem.write(0xFF01, 0x05);

        cpu.tick(&mut mem);
        assert_eq!(2, cpu.cycles);
        assert!(cpu.read_flag(FL_CARRY));
        assert!(!cpu.read_flag(FL_ZERO));
    }
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
