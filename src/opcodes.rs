use crate::types::*;

pub const OP_NOP: Byte = 0xEA;

pub const OP_LDA_IMM: Byte = 0xA9;
pub const OP_LDA_ZP: Byte = 0xA5;
pub const OP_LDA_ZPX: Byte = 0xB5;
pub const OP_LDA_ABS: Byte = 0xAD;
pub const OP_LDA_ABX: Byte = 0xBD;
pub const OP_LDA_ABY: Byte = 0xB9;
pub const OP_LDA_IDX: Byte = 0xA1;
pub const OP_LDA_IDY: Byte = 0xB1;

pub const OP_INC_ABS: Byte = 0xEE;
