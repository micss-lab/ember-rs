pub const BASE_MSG_ID: u8 = 0xFE;
/// Reserved for future multi-hop support (an `ExtEnvelope` records each relay's
/// `ReceivedObject`). A direct point-to-point channel is single-hop, so it is unused for now.
#[allow(dead_code)]
pub const EXT_MSG_ID: u8 = 0xFD;

pub const END_OF_COLLECTION: u8 = 0x01;

pub const PARAM_TO: u8 = 0x02;
pub const PARAM_FROM: u8 = 0x03;
pub const PARAM_ACL_REPRESENTATION: u8 = 0x04;
pub const PARAM_COMMENTS: u8 = 0x05;
pub const PARAM_PAYLOAD_LENGTH: u8 = 0x06;
pub const PARAM_PAYLOAD_ENCODING: u8 = 0x07;
pub const PARAM_INTENDED_RECEIVER: u8 = 0x09;
pub const PARAM_RECEIVED: u8 = 0x0a;
pub const PARAM_TRANSPORT_BEHAVIOUR: u8 = 0x0b;
pub const PARAM_USER_DEFINED: u8 = 0x00;

pub const ACL_REP_BIT_EFFICIENT: u8 = 0x10;
pub const ACL_REP_STRING: u8 = 0x11;
pub const ACL_REP_XML: u8 = 0x12;
pub const ACL_REP_USER_DEFINED: u8 = 0x00;

pub const AID_TAG: u8 = 0x02;
pub const AID_ADDRESSES: u8 = 0x02;
pub const AID_RESOLVERS: u8 = 0x03;
pub const AID_USER_DEFINED: u8 = 0x05;

pub const RECEIVED_FROM: u8 = 0x02;
pub const RECEIVED_ID: u8 = 0x03;
pub const RECEIVED_VIA: u8 = 0x04;

pub const DATE_ABS: u8 = 0x20;
pub const DATE_REL_POS: u8 = 0x21;
pub const DATE_REL_NEG: u8 = 0x22;
pub const DATE_ABS_TD: u8 = 0x24;
pub const DATE_REL_POS_TD: u8 = 0x25;
pub const DATE_REL_NEG_TD: u8 = 0x26;

pub const ANY_STRING: u8 = 0x14;
pub const ANY_BYTES_8: u8 = 0x16;
pub const ANY_BYTES_16: u8 = 0x17;
pub const ANY_BYTES_32: u8 = 0x19;

pub const NUM_DECIMAL: u8 = 0x12;
pub const NUM_HEX: u8 = 0x13;
