mod encode;
mod parser;

use crate::message::Message;

pub fn encode(message: &Message) -> alloc::vec::Vec<u8> {
    encode::encode(message)
}

pub fn decode(bytes: &[u8]) -> Result<Message, ()> {
    parser::messsage::message(&bstr::BStr::new(bytes).into())
        .map_err(|e| log::error!("failed to parse string repr: {e}"))
}
