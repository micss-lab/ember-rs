use alloc::vec::Vec;

use crate::message::Message;

use super::builder;

mod decode;
mod encode;

pub fn encode(message: &Message) -> alloc::vec::Vec<u8> {
    let mut result = Vec::new();
    encode::encode(message, &mut result)
        .map(|_| result)
        .unwrap_or_else(|e| {
            log::error!("failed to encode acl using string representation: {e}");
            Vec::with_capacity(0)
        })
}

pub fn decode(bytes: &[u8]) -> Result<Message, ()> {
    decode::messsage::message(&bstr::BStr::new(bytes).into())
        .map_err(|e| log::error!("failed to parse string repr: {e}"))
}
