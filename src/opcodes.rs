use crate::types::*;

pub const OP_NOP: Byte = 0xEA;

pub const OP_JMP_ABS: Byte = 0x4C;
pub const OP_JMP_IND: Byte = 0x6C;

pub const OP_LDA_IMM: Byte = 0xA9;
pub const OP_LDA_ZP0: Byte = 0xA5;
pub const OP_LDA_ZPX: Byte = 0xB5;
pub const OP_LDA_ABS: Byte = 0xAD;
pub const OP_LDA_ABX: Byte = 0xBD;
pub const OP_LDA_ABY: Byte = 0xB9;
pub const OP_LDA_IDX: Byte = 0xA1;
pub const OP_LDA_IDY: Byte = 0xB1;

pub const OP_LDX_IMM: Byte = 0xA2;
pub const OP_LDX_ZP0: Byte = 0xA6;
pub const OP_LDX_ZPY: Byte = 0xB6;
pub const OP_LDX_ABS: Byte = 0xAE;
pub const OP_LDX_ABY: Byte = 0xBE;

pub const OP_INC_ZP0: Byte = 0xE6;
pub const OP_INC_ZPX: Byte = 0xF6;
pub const OP_INC_ABS: Byte = 0xEE;
pub const OP_INC_ABX: Byte = 0xFE;

pub const OP_INX_IMP: Byte = 0xE8;

pub const OP_INY_IMP: Byte = 0xC8;
