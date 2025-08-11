use alloc::string::String;
use serde::Serialize;

use crate::message::Message;

mod parser;
mod ser;

pub fn to_string<T>(value: &T) -> String
where
    T: Serialize,
{
    let mut serializer = self::ser::StringSerializer::default();
    value
        .serialize(&mut serializer)
        .expect("failed to serialize");
    serializer.into_string()
}

pub fn try_from_bytes(bytes: impl AsRef<[u8]>) -> Result<Message, ()> {
    let bytes = bytes.as_ref();
    self::parser::messsage::message(&bstr::BStr::new(bytes).into())
        .map_err(|e| log::error!("failed to parse human readable representation: {e}"))
}
