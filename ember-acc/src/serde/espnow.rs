pub mod ser {
    use ember_core::agent::aid::Aid;
    use ember_core::message::MessageEnvelope;

    #[derive(serde::Serialize)]
    struct EspNowEnvelopeSer<'a> {
        to: &'a [Aid],
        from: &'a Aid,
    }

    #[derive(serde::Serialize)]
    pub struct EspNowMessageSer<'a> {
        envelope: EspNowEnvelopeSer<'a>,
        content: &'a [u8],
    }

    impl<'a> EspNowMessageSer<'a> {
        pub fn new(envelope: &'a MessageEnvelope, content: &'a [u8]) -> Self {
            Self {
                envelope: EspNowEnvelopeSer {
                    to: &envelope.to,
                    from: &envelope.from,
                },
                content,
            }
        }
    }
}

pub mod de {
    use alloc::vec::Vec;

    use ember_core::agent::aid::Aid;
    use ember_core::message::repr;
    use ember_core::message::{
        AclRepresentation, MessageEnvelope, MessageEnvelopes, Payload, TransportMessage,
    };

    #[derive(serde::Deserialize)]
    struct EspNowEnvelopeDe {
        to: Vec<Aid>,
        from: Aid,
    }

    #[derive(serde::Deserialize)]
    pub struct EspNowMessageDe<'a> {
        envelope: EspNowEnvelopeDe,
        content: &'a [u8],
    }

    impl<'a> EspNowMessageDe<'a> {
        pub fn into_transport(self) -> Result<TransportMessage, ()> {
            let message = repr::payload::string::decode(self.content)?;
            let envelope = MessageEnvelope {
                to: self.envelope.to,
                from: self.envelope.from,
                date: chrono::DateTime::<chrono::Utc>::MIN_UTC.into(),
                acl_representation: AclRepresentation::String,
                other: None,
            };
            Ok(TransportMessage {
                envelopes: MessageEnvelopes {
                    base: envelope,
                    others: Vec::with_capacity(0),
                },
                payload: Payload::AclMessage(message),
            })
        }
    }
}
