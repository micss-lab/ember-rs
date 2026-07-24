use alloc::vec::Vec;

use crate::message::MessageEnvelope;

use self::codec::*;

mod codec;
mod decode;
mod encode;

/// Encodes just the envelope: `BaseMsgId EnvLen ACLRepresentation Date (Parameter)*
/// EndOfEnvelope`. Per the spec, the ACL payload bytes belong immediately after this, which
/// callers are responsible for appending (see [`decode`]).
pub fn encode(envelope: &MessageEnvelope) -> Vec<u8> {
    let mut body = Vec::new();

    encode::push_acl_representation(&envelope.acl_representation, &mut body);
    encode::push_date(&envelope.date, &mut body);

    if !envelope.to.is_empty() {
        body.push(PARAM_TO);
        encode::push_agent_identifier_sequence(&envelope.to, &mut body);
    }

    body.push(PARAM_FROM);
    encode::push_aid(&envelope.from, &mut body);

    if let Some(other) = &envelope.other {
        for (key, value) in other {
            body.push(PARAM_USER_DEFINED);
            encode::push_null_terminated_string(key.as_bytes(), &mut body);
            encode::push_null_terminated_string(value, &mut body);
        }
    }

    body.push(END_OF_COLLECTION);

    let mut out = Vec::with_capacity(body.len() + 3);
    out.push(BASE_MSG_ID);
    match u16::try_from(3 + body.len()) {
        Ok(len) => out.extend_from_slice(&len.to_be_bytes()),
        Err(_) => {
            // JumboEnvelope: EmptyLen16 (0x00 0x00) followed by a 4-byte length.
            out.extend_from_slice(&[0x00, 0x00]);
            out.extend_from_slice(&((7 + body.len()) as u32).to_be_bytes());
        }
    }
    out.extend_from_slice(&body);
    out
}

/// Decodes an envelope from the front of `bytes` and returns it along with whatever bytes follow
/// (the ACL payload, per the spec's `MessageEnvelope = (ExtEnvelope)* BaseEnvelope Payload`).
pub fn decode(bytes: &[u8]) -> Result<(MessageEnvelope, &[u8]), ()> {
    let envelope_len = read_envelope_len(bytes)?;

    if bytes.len() < envelope_len {
        log::error!(
            "truncated bit-efficient envelope: expected at least {envelope_len} bytes, got {}",
            bytes.len()
        );
        return Err(());
    }
    let (env_bytes, rest) = bytes.split_at(envelope_len);

    let input = crate::util::parsing::BStr::from(bstr::BStr::new(env_bytes));
    let envelope = decode::parser::envelope(&input)
        .map_err(|e| log::error!("bit-efficient envelope decode error: {e}"))?;

    Ok((envelope, rest))
}

fn read_envelope_len(bytes: &[u8]) -> Result<usize, ()> {
    if bytes.len() < 3 || bytes[0] != BASE_MSG_ID {
        log::error!("input does not start with a bit-efficient envelope");
        return Err(());
    }

    let len16 = u16::from_be_bytes([bytes[1], bytes[2]]);
    if len16 != 0 {
        return Ok(len16 as usize);
    }

    let Some(len32_bytes) = bytes.get(3..7) else {
        log::error!("truncated jumbo envelope length");
        return Err(());
    };
    Ok(u32::from_be_bytes(len32_bytes.try_into().expect("slice has length 4")) as usize)
}

#[cfg(all(test, not(target_os = "none")))]
mod round_trip_tests {
    extern crate std;

    use alloc::collections::BTreeMap;

    use crate::agent::aid::Aid;
    use crate::message::{AclRepresentation, MessageEnvelope};

    use super::{decode, encode};

    fn aid(s: &str) -> Aid {
        s.parse().unwrap()
    }

    fn date(
        y: i32,
        mo: u32,
        d: u32,
        h: u32,
        mi: u32,
        s: u32,
        ms: u32,
    ) -> chrono::DateTime<chrono::FixedOffset> {
        chrono::NaiveDate::from_ymd_opt(y, mo, d)
            .unwrap()
            .and_hms_milli_opt(h, mi, s, ms)
            .unwrap()
            .and_utc()
            .into()
    }

    fn round_trip(envelope: MessageEnvelope) {
        let encoded = encode(&envelope);
        let (decoded, rest) = decode(&encoded).expect("decode failed");
        assert!(rest.is_empty());
        assert_eq!(envelope, decoded);
    }

    #[test]
    fn minimal_envelope() {
        round_trip(MessageEnvelope {
            to: alloc::vec::Vec::new(),
            from: aid("sender@local"),
            date: date(1970, 1, 1, 0, 0, 0, 0),
            acl_representation: AclRepresentation::BitEfficient,
            other: None,
        });
    }

    #[test]
    fn multiple_recipients_and_other_params() {
        round_trip(MessageEnvelope {
            to: alloc::vec::Vec::from([aid("a@device-1"), aid("b@device-2")]),
            from: aid("sender@local"),
            date: date(2024, 3, 15, 12, 30, 45, 678),
            acl_representation: AclRepresentation::String,
            other: Some(BTreeMap::from([(
                "X-Ember-Hop-Count".into(),
                bstr::BString::from(alloc::vec![0x01]),
            )])),
        });
    }

    #[test]
    fn user_defined_acl_representation() {
        round_trip(MessageEnvelope {
            to: alloc::vec::Vec::from([aid("a@device-1")]),
            from: aid("sender@local"),
            date: date(2024, 3, 15, 12, 30, 45, 0),
            acl_representation: AclRepresentation::Other("x-ember-custom".into()),
            other: None,
        });
    }

    #[test]
    fn envelope_precedes_acl_payload_on_the_wire() {
        use crate::message::content::fipa_sl::Sl0Content;
        use crate::message::repr::payload;
        use crate::message::{Content, Message, Performative, Receiver};

        let envelope = MessageEnvelope {
            to: alloc::vec::Vec::from([aid("b@device-2")]),
            from: aid("a@device-1"),
            date: date(2024, 3, 15, 12, 30, 45, 0),
            acl_representation: AclRepresentation::BitEfficient,
            other: None,
        };
        let message = Message {
            performative: Performative::Inform,
            receiver: Some(Receiver::Single(aid("b@device-2"))),
            ontology: None,
            other: None,
            content: Some(Content::FipaSl0(
                Sl0Content::try_from_sl("(some_proposition)").unwrap(),
            )),
        };

        let mut wire = encode(&envelope);
        wire.extend(payload::bit_efficient::encode(&message));

        let (decoded_envelope, rest) = decode(&wire).expect("envelope decode failed");
        assert_eq!(envelope, decoded_envelope);

        let decoded_message = payload::bit_efficient::decode(rest).expect("payload decode failed");
        assert_eq!(message, decoded_message);
    }

    #[test]
    fn tolerates_unknown_parameters() {
        // Hand-crafted: BaseMsgId, Len16 (computed below), ACLRepresentation (bit-efficient),
        // Date (1970-01-01T00:00:00.000), a `comments` parameter (0x05) that the encoder never
        // emits, a `from` parameter, then EndOfEnvelope.
        let mut body = alloc::vec::Vec::new();
        body.push(0x10); // ACLRepresentation::BitEfficient
        // absolute date, 1970-01-01T00:00:00.000: tag(1) + year(2) + month/day/h/m/s(5) + ms(2)
        body.extend_from_slice(&[0x20, 0x07, 0xB2, 0x01, 0x01, 0, 0, 0, 0, 0]);
        body.push(0x05); // PARAM_COMMENTS
        body.extend_from_slice(b"hello\0");
        body.push(0x03); // PARAM_FROM
        body.push(0x02); // AID_TAG
        body.extend_from_slice(b"a@device-1\0");
        body.push(0x01); // EndOfCollection (AID)
        body.push(0x01); // EndOfEnvelope

        let mut wire = alloc::vec::Vec::new();
        wire.push(0xFE);
        wire.extend_from_slice(&(3 + body.len() as u16).to_be_bytes());
        wire.extend_from_slice(&body);

        let (envelope, rest) = decode(&wire).expect("decode should tolerate unknown `comments`");
        assert!(rest.is_empty());
        assert_eq!(envelope.from, aid("a@device-1"));
        assert!(envelope.other.is_none());
    }
}
