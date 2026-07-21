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

#[cfg(all(test, not(target_os = "none")))]
mod round_trip_tests {
    use crate::agent::aid::Aid;
    use crate::message::{Content, Message, Performative, Receiver};

    use super::{decode, encode};

    fn aid(s: &str) -> Aid {
        s.parse().unwrap()
    }

    fn round_trip(msg: Message) {
        let encoded = encode(&msg);
        let decoded = decode(&encoded).expect("decode failed");
        assert_eq!(msg, decoded);
    }

    #[test]
    fn inform_bytes_content_is_base64_round_tripped() {
        round_trip(Message {
            performative: Performative::Inform,
            receiver: Some(Receiver::Single(aid("bob@local"))),
            ontology: None,
            other: None,
            content: Some(Content::Bytes(alloc::vec![0xDE, 0xAD, 0xBE, 0xEF])),
        });
    }
}
