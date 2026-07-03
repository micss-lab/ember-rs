pub const MAGIC: [u8; 2] = [0xCA, 0xED];

pub const VER_0_1_0: u8 = 0x40;
pub const VER_EXPLICIT: u8 = 0x41;

pub const EXPR_LIT_POS: u8 = 0x10;
pub const EXPR_LIT_NEG: u8 = 0x11;

pub const WORD: u8 = 0x30;

pub const END: u8 = 0x00;
pub const T_INT: u8 = 0x20;
pub const T_FLT: u8 = 0x21;
pub const T_STR: u8 = 0x22;
pub const T_LIT_POS: u8 = 0x23;
pub const T_LIT_NEG: u8 = 0x24;
pub const T_VAR: u8 = 0x25;
