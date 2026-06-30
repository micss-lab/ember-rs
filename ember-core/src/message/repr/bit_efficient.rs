mod codec;
mod decode;
mod encode;

use crate::message::Message;

pub fn encode(message: &Message) -> alloc::vec::Vec<u8> {
    let mut result = alloc::vec::Vec::new();
    encode::encode(message, &mut result);
    result
}

pub fn decode(bytes: &[u8]) -> Result<Message, ()> {
    decode::decode(bytes)
}

#[cfg(all(test, not(target_os = "none")))]
mod round_trip_tests {
    extern crate std;

    use alloc::collections::BTreeSet;

    use crate::agent::aid::Aid;
    use crate::message::content::fipa_sl::Sl0Content as SlContent;
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
    fn inform_single_receiver_structured_content() {
        let content = SlContent::try_from_sl("(some_proposition)").unwrap();
        round_trip(Message {
            performative: Performative::Inform,
            sender: Some(aid("alice@local")),
            receiver: Receiver::Single(aid("bob@local")),
            reply_to: None,
            ontology: Some("test-ontology".into()),
            content: Content::FipaSl0(content),
        });
    }

    #[test]
    fn request_multiple_receivers() {
        round_trip(Message {
            performative: Performative::Request,
            sender: None,
            receiver: Receiver::Multiple(BTreeSet::from([
                aid("a@local"),
                aid("b@remote-platform"),
            ])),
            reply_to: None,
            ontology: None,
            content: Content::Bytes(alloc::vec![0xDE, 0xAD, 0xBE, 0xEF]),
        });
    }

    #[test]
    fn cfp_no_sender_no_ontology() {
        let content = SlContent::try_from_sl("(true)").unwrap();
        round_trip(Message {
            performative: Performative::Cfp,
            sender: None,
            receiver: Receiver::Single(aid("agent@local")),
            reply_to: None,
            ontology: None,
            content: Content::FipaSl0(content),
        });
    }

    #[test]
    fn all_performatives_encode_decode() {
        use Performative::*;
        let perfs = [
            AcceptProposal,
            Agree,
            Cancel,
            Cfp,
            Confirm,
            Disconfirm,
            Failure,
            Inform,
            InformIf,
            InformRef,
            NotUnderstood,
            Propose,
            QueryIf,
            QueryRef,
            Refuse,
            RejectProposal,
            Request,
            RequestWhen,
            RequestWhenever,
            Subscribe,
            Proxy,
            Propagate,
        ];
        let content = SlContent::try_from_sl("(p)").unwrap();
        for perf in perfs {
            let msg = Message {
                performative: perf,
                sender: None,
                receiver: Receiver::Single(aid("x@local")),
                reply_to: None,
                ontology: None,
                content: Content::FipaSl0(content.clone()),
            };
            let encoded = encode(&msg);
            let decoded = decode(&encoded).expect("decode failed");
            assert_eq!(decoded.performative, perf);
        }
    }
}
