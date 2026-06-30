use alloc::vec::Vec;

use crate::message::Message;

mod decode;
mod encode;

pub fn encode(message: &Message) -> alloc::vec::Vec<u8> {
    let mut result = Vec::new();
    encode::encode(message, &mut result).expect("failed to encode acl using string representation");
    result
}

pub fn decode(bytes: &[u8]) -> Result<Message, ()> {
    decode::messsage::message(&bstr::BStr::new(bytes).into())
        .map_err(|e| log::error!("failed to parse string repr: {e}"))
}
