pub const MESSAGE_ID: u8 = 0xFA;

pub const VERSION_1_0: u8 = 0x10;

pub const END_OF_COLLECTION: u8 = 0x01;

pub const PERF_ACCEPT_PROPOSAL: u8 = 0x01;
pub const PERF_AGREE: u8 = 0x02;
pub const PERF_CANCEL: u8 = 0x03;
pub const PERF_CFP: u8 = 0x04;
pub const PERF_CONFIRM: u8 = 0x05;
pub const PERF_DISCONFIRM: u8 = 0x06;
pub const PERF_FAILURE: u8 = 0x07;
pub const PERF_INFORM: u8 = 0x08;
pub const PERF_INFORM_IF: u8 = 0x09;
pub const PERF_INFORM_REF: u8 = 0x0A;
pub const PERF_NOT_UNDERSTOOD: u8 = 0x0B;
pub const PERF_PROPAGATE: u8 = 0x0C;
pub const PERF_PROPOSE: u8 = 0x0D;
pub const PERF_PROXY: u8 = 0x0E;
pub const PERF_QUERY_IF: u8 = 0x0F;
pub const PERF_QUERY_REF: u8 = 0x10;
pub const PERF_REFUSE: u8 = 0x11;
pub const PERF_REJECT_PROPOSAL: u8 = 0x12;
pub const PERF_REQUEST: u8 = 0x13;
pub const PERF_REQUEST_WHEN: u8 = 0x14;
pub const PERF_REQUEST_WHENEVER: u8 = 0x15;
pub const PERF_SUBSCRIBE: u8 = 0x16;

pub const KW_SENDER: u8 = 0x02;
pub const KW_RECEIVER: u8 = 0x03;
pub const KW_CONTENT: u8 = 0x04;
pub const KW_REPLY_WITH: u8 = 0x05;
pub const KW_REPLY_BY: u8 = 0x06;
pub const KW_IN_REPLY_TO: u8 = 0x07;
pub const KW_REPLY_TO: u8 = 0x08;
pub const KW_LANGUAGE: u8 = 0x09;
pub const KW_ENCODING: u8 = 0x0A;
pub const KW_ONTOLOGY: u8 = 0x0B;
pub const KW_PROTOCOL: u8 = 0x0C;
pub const KW_CONVERSATION_ID: u8 = 0x0D;

pub const AID_TAG_ADDRESSES: u8 = 0x02;
pub const AID_TAG_RESOLVERS: u8 = 0x03;
pub const AID_TAG_USER_DEF: u8 = 0x04;

pub const BIN_WORD: u8 = 0x10;
pub const BIN_WORD_IDX: u8 = 0x11;

pub const BIN_STR_NULL: u8 = 0x14;
pub const BIN_STR_IDX: u8 = 0x15;
pub const BIN_STR_8: u8 = 0x16;
pub const BIN_STR_16: u8 = 0x17;
pub const BIN_STR_IDX2: u8 = 0x18;
pub const BIN_STR_32: u8 = 0x19;
