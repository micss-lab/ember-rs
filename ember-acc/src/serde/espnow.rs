pub mod ser {
    use ember_core::message::{MessageEnvelope, MessageKind};

    pub struct EspNowMessageSer<'a>(pub &'a MessageEnvelope);
    struct EspNowEnvelopeSer<'a>(&'a MessageEnvelope);

    impl serde::Serialize for EspNowMessageSer<'_> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            use alloc::string::ToString;
            use serde::ser::SerializeStruct;

            let mut message = serializer.serialize_struct("message", 2)?;
            message.serialize_field("envelope", &EspNowEnvelopeSer(self.0))?;
            match &self.0.message {
                MessageKind::Parsed(m) => {
                    message.serialize_field("content", m.to_string().as_bytes())?
                }
            }
            message.end()
        }
    }

    impl serde::Serialize for EspNowEnvelopeSer<'_> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            use serde::ser::SerializeStruct;

            let mut envelope = serializer.serialize_struct("envelope", 2)?;
            envelope.serialize_field("to", &self.0.to)?;
            envelope.serialize_field("from", &self.0.from)?;
            envelope.end()
        }
    }
}

pub mod de {
    use alloc::collections::BTreeMap;
    use alloc::vec::Vec;

    use serde::de::Unexpected;

    use ember_core::agent::aid::Aid;
    use ember_core::message::{AclRepresentation, Message, MessageEnvelope, MessageKind};

    pub struct EspNowMessageDe {
        envelope: EspNowEnvelopeDe,
        content: EspNowContentDe,
    }
    struct EspNowEnvelopeDe {
        to: Vec<Aid>,
        from: Option<Aid>,
    }
    struct EspNowContentDe {
        message: Message,
    }

    impl<'de> serde::Deserialize<'de> for EspNowMessageDe {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct Visitor;

            impl<'de> serde::de::Visitor<'de> for Visitor {
                type Value = EspNowMessageDe;

                fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                    formatter.write_str("struct message")
                }

                fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
                {
                    // In [`postcard`], structs are visited as sequences with known amount of fields
                    // and order. In fact, the `deserialize_struct` implementation forwards to
                    // `deserialize_tuple`.
                    let envelope = seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                    let content = seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                    Ok(EspNowMessageDe { envelope, content })
                }
            }

            const FIELDS: &[&str] = &["envelope", "content"];
            deserializer.deserialize_struct("message", FIELDS, Visitor)
        }
    }

    impl<'de> serde::Deserialize<'de> for EspNowEnvelopeDe {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct Visitor;

            impl<'de> serde::de::Visitor<'de> for Visitor {
                type Value = EspNowEnvelopeDe;

                fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                    formatter.write_str("struct envelope")
                }

                fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
                {
                    // In [`postcard`], structs are visited as sequences with known amount of fields
                    // and order. In fact, the `deserialize_struct` implementation forwards to
                    // `deserialize_tuple`.
                    let to = seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                    let from = seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                    Ok(EspNowEnvelopeDe { to, from })
                }
            }

            const FIELDS: &[&str] = &["to", "from"];
            deserializer.deserialize_struct("envelope", FIELDS, Visitor)
        }
    }

    impl<'de> serde::Deserialize<'de> for EspNowContentDe {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct Visitor;

            impl<'de> serde::de::Visitor<'de> for Visitor {
                type Value = EspNowContentDe;

                fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                    formatter.write_str("struct content")
                }

                fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    Ok(EspNowContentDe {
                        message: Message::try_from_bytes(v)
                            .map_err(|_| E::invalid_value(Unexpected::Bytes(v), &self))?,
                    })
                }
            }

            deserializer.deserialize_bytes(Visitor)
        }
    }

    impl EspNowMessageDe {
        pub fn into_envelope(self) -> MessageEnvelope {
            MessageEnvelope {
                to: self.envelope.to,
                from: self.envelope.from,
                date: chrono::DateTime::<chrono::Utc>::MIN_UTC.into(),
                acl_representation: AclRepresentation::BitEfficient,
                parameters: BTreeMap::new(),
                message: MessageKind::Parsed(self.content.message),
            }
        }
    }
}
