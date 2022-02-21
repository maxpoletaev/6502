use crate::types::*;

// NOP - No Operation
pub const OP_NOP: Byte = 0xEA;

// JMP - Jump to Memory Location
pub const OP_JMP_ABS: Byte = 0x4C;
pub const OP_JMP_IND: Byte = 0x6C;

// JSR - Jump to Subroutine
pub const OP_JSR_ABS: Byte = 0x20;

// RTS - Return from Subroutine
pub const OP_RTS_IMP: Byte = 0x60;

// LDA - Load Accumulator
pub const OP_LDA_IMM: Byte = 0xA9;
pub const OP_LDA_ZP0: Byte = 0xA5;
pub const OP_LDA_ZPX: Byte = 0xB5;
pub const OP_LDA_ABS: Byte = 0xAD;
pub const OP_LDA_ABX: Byte = 0xBD;
pub const OP_LDA_ABY: Byte = 0xB9;
pub const OP_LDA_IDX: Byte = 0xA1;
pub const OP_LDA_IDY: Byte = 0xB1;

// LDX - Load X Register
pub const OP_LDX_IMM: Byte = 0xA2;
pub const OP_LDX_ZP0: Byte = 0xA6;
pub const OP_LDX_ZPY: Byte = 0xB6;
pub const OP_LDX_ABS: Byte = 0xAE;
pub const OP_LDX_ABY: Byte = 0xBE;

// LDY - Load Y Register
pub const OP_LDY_IMM: Byte = 0xA0;
pub const OP_LDY_ZP0: Byte = 0xA4;
pub const OP_LDY_ZPX: Byte = 0xB4;
pub const OP_LDY_ABS: Byte = 0xAC;
pub const OP_LDY_ABX: Byte = 0xBC;

// STA - Store Accumulator
pub const OP_STA_ZP0: Byte = 0x85;
pub const OP_STA_ZPX: Byte = 0x95;
pub const OP_STA_ABS: Byte = 0x8D;
pub const OP_STA_ABX: Byte = 0x9D;
pub const OP_STA_ABY: Byte = 0x99;
pub const OP_STA_IDX: Byte = 0x81;
pub const OP_STA_IDY: Byte = 0x91;

// STX - Store X Register
pub const OP_STX_ZP0: Byte = 0x86;
pub const OP_STX_ZPY: Byte = 0x96;
pub const OP_STX_ABS: Byte = 0x8E;

// STY - Store Y Register
pub const OP_STY_ZP0: Byte = 0x84;
pub const OP_STY_ZPX: Byte = 0x94;
pub const OP_STY_ABS: Byte = 0x8C;

// INC - Increment Memory
pub const OP_INC_ZP0: Byte = 0xE6;
pub const OP_INC_ZPX: Byte = 0xF6;
pub const OP_INC_ABS: Byte = 0xEE;
pub const OP_INC_ABX: Byte = 0xFE;

// INX - Increment X Register
pub const OP_INX_IMP: Byte = 0xE8;

// INY - Increment Y Register
pub const OP_INY_IMP: Byte = 0xC8;

// ADC - Add with Carry
pub const OP_ADC_IMM: Byte = 0x69;
pub const OP_ADC_ZP0: Byte = 0x65;
pub const OP_ADC_ZPX: Byte = 0x75;
pub const OP_ADC_ABS: Byte = 0x6D;
pub const OP_ADC_ABX: Byte = 0x7D;
pub const OP_ADC_ABY: Byte = 0x79;
pub const OP_ADC_IDX: Byte = 0x61;
pub const OP_ADC_IDY: Byte = 0x71;

// CLC - Clear Carry Flag
pub const OP_CLC_IMP: Byte = 0x18;

// BCC - Branch if Carry Clear
pub const OP_BCC_REL: Byte = 0x90;

// BCS - Branch if Carry Set
pub const OP_BCS_REL: Byte = 0xB0;

// BEQ - Branch if Equal
pub const OP_BEQ_REL: Byte = 0xF0;

// BNE - Branch if Not Equal
pub const OP_BNE_REL: Byte = 0xD0;

// BMI - Branch if Minus
pub const OP_BMI_REL: Byte = 0x30;

// BPL - Branch if Positive
pub const OP_BPL_REL: Byte = 0x10;

// BVC - Branch if Overflow Clear
pub const OP_BVC_REL: Byte = 0x50;

// BVS - Branch if Overflow Set
pub const OP_BVS_REL: Byte = 0x70;

// TAX - Transfer Accumulator to X
pub const OP_TAX_IMP: Byte = 0xAA;

// TXA - Transfer X to Accumulator
pub const OP_TXA_IMP: Byte = 0x8A;

// TAY - Transfer Accumulator to Y
pub const OP_TAY_IMP: Byte = 0xA8;

// TYA - Transfer Y to Accumulator
pub const OP_TYA_IMP: Byte = 0x98;

// TSX - Transfer Stack Pointer to X
pub const OP_TSX_IMP: Byte = 0xBA;

// TXS - Transfer X to Stack Pointer
pub const OP_TXS_IMP: Byte = 0x9A;

// CMP - Compare
pub const OP_CMP_IMM: Byte = 0xC9;
pub const OP_CMP_ZP0: Byte = 0xC5;
pub const OP_CMP_ZPX: Byte = 0xD5;
pub const OP_CMP_ABS: Byte = 0xCD;
pub const OP_CMP_ABX: Byte = 0xDD;
pub const OP_CMP_ABY: Byte = 0xD9;
pub const OP_CMP_IDX: Byte = 0xC1;
pub const OP_CMP_IDY: Byte = 0xD1;

// CPX - Compare X Register
pub const OP_CPX_IMM: Byte = 0xE0;
pub const OP_CPX_ZP0: Byte = 0xE4;
pub const OP_CPX_ABS: Byte = 0xEC;

// CPY - Compare Y Register
pub const OP_CPY_IMM: Byte = 0xC0;
pub const OP_CPY_ZP0: Byte = 0xC4;
pub const OP_CPY_ABS: Byte = 0xCC;

// PHA - Push Accumulator
pub const OP_PHA_IMP: Byte = 0x48;

// PHP - Push Processor Status
pub const OP_PHP_IMP: Byte = 0x08;

// PLA - Pull Accumulator
pub const OP_PLA_IMP: Byte = 0x68;

// PLP - Pull Processor Status
pub const OP_PLP_IMP: Byte = 0x28;
