use alloc::vec::Vec;

use crate::message::Message;

use super::builder;

use self::codec::*;

mod codec;
mod decode;
mod encode;

pub fn encode(message: &Message) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(MESSAGE_ID);
    out.push(VERSION_1_0);
    out.push(encode::performative_code(message.performative));

    if let Some(ref receiver) = message.receiver {
        out.push(KW_RECEIVER);
        encode::push_recipient_expr(receiver, &mut out);
    }

    if let Some(ontology) = &message.ontology {
        out.push(KW_ONTOLOGY);
        encode::push_bin_word(ontology.as_bytes(), &mut out);
    }

    if let Some(ref content) = message.content {
        encode::push_content_and_language(content, &mut out);
    }

    out.push(END_OF_COLLECTION);
    out
}

pub fn decode(bytes: &[u8]) -> Result<Message, ()> {
    let input = crate::util::parsing::BStr::from(bstr::BStr::new(bytes));
    decode::parser::message(&input).map_err(|e| log::error!("bit-efficient decode error: {e}"))
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
            receiver: Some(Receiver::Single(aid("bob@local"))),
            ontology: Some("test-ontology".into()),
            other: None,
            content: Some(Content::FipaSl0(content)),
        });
    }

    #[test]
    fn request_multiple_receivers() {
        round_trip(Message {
            performative: Performative::Request,
            receiver: Some(Receiver::Multiple(BTreeSet::from([
                aid("a@local"),
                aid("b@remote-platform"),
            ]))),
            ontology: None,
            other: None,
            content: Some(Content::Bytes(alloc::vec![0xDE, 0xAD, 0xBE, 0xEF])),
        });
    }

    #[test]
    fn cfp_no_ontology() {
        let content = SlContent::try_from_sl("(true)").unwrap();
        round_trip(Message {
            performative: Performative::Cfp,
            receiver: Some(Receiver::Single(aid("agent@local"))),
            ontology: None,
            other: None,
            content: Some(Content::FipaSl0(content)),
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
                receiver: Some(Receiver::Single(aid("x@local"))),
                ontology: None,
                other: None,
                content: Some(Content::FipaSl0(content.clone())),
            };
            let encoded = encode(&msg);
            let decoded = decode(&encoded).expect("decode failed");
            assert_eq!(decoded.performative, perf);
        }
    }
}
