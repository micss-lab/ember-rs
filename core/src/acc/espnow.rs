pub(super) use esp_wifi::esp_now::{EspNowReceiver, EspNowSender};

use alloc::collections::btree_map::BTreeMap;
use alloc::string::ToString;
use alloc::vec::Vec;

use esp_wifi::esp_now::{EspNowReceiver as Receiver, EspNowSender as Sender};
use macaddr::MacAddr6;
use serde::ser::SerializeStruct;

use crate::acl::message::{AclRepresentation, Message, MessageEnvelope, MessageKind};
use crate::agent::Aid;

use super::Acc;

pub(super) struct EspNowChannel<'c> {
    sender: Option<Sender<'c>>,
    receiver: Option<Receiver<'c>>,
}

impl<'c> EspNowChannel<'c> {
    pub(super) fn new(sender: Option<Sender<'c>>, receiver: Option<Receiver<'c>>) -> Self {
        Self { sender, receiver }
    }
}

impl<'c> Acc for EspNowChannel<'c> {
    fn send(&mut self, address: &Aid, message: MessageEnvelope) -> Result<(), ()> {
        let Some(sender) = self.sender.as_mut() else {
            log::error!("EspNow channel is not configured for sending messages.");
            return Err(());
        };

        if let Err(err) = sender
            .send(
                &aid_to_mac(address),
                &postcard::to_allocvec(&EspNowMessageSer(&message))
                    .expect("failed to serialize message into postcard data format"),
            )
            .and_then(|w| w.wait())
        {
            log::error!("EspNow send error: {:?}", err);
        }
        Ok(())
    }

    fn receive(&mut self) -> Option<MessageEnvelope> {
        let message = self.receiver.as_mut().and_then(|r| r.receive())?;
        let envelope = postcard::from_bytes::<EspNowMessageDe>(message.data())
            .expect("failed to deserialize data into envelope")
            .into_envelope();
        Some(envelope)
    }
}

struct EspNowMessageSer<'a>(&'a MessageEnvelope);
struct EspNowEnvelopeSer<'a>(&'a MessageEnvelope);

struct EspNowMessageDe {
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

impl serde::Serialize for EspNowMessageSer<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut message = serializer.serialize_struct("message", 2)?;
        message.serialize_field("envelope", &EspNowEnvelopeSer(self.0))?;
        match &self.0.message {
            MessageKind::Structured(m) => {
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
        let mut envelope = serializer.serialize_struct("envelope", 2)?;
        envelope.serialize_field("to", &self.0.to)?;
        envelope.serialize_field("from", &self.0.from)?;
        envelope.end()
    }
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
                formatter.write_str("struct envelope")
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
                        .expect("failed to parse content as acl message"),
                })
            }
        }

        deserializer.deserialize_bytes(Visitor)
    }
}

impl EspNowMessageDe {
    fn into_envelope(self) -> MessageEnvelope {
        MessageEnvelope {
            to: self.envelope.to,
            from: self.envelope.from,
            date: chrono::DateTime::<chrono::Utc>::MIN_UTC.into(),
            acl_representation: AclRepresentation::BitEfficient,
            parameters: BTreeMap::new(),
            message: MessageKind::Structured(self.content.message),
        }
    }
}

fn aid_to_mac(aid: &Aid) -> [u8; 6] {
    use crate::agent::AgentPlatform::*;
    let mac = match aid.platform() {
        Local => panic!("espnow channel does not support sending messages to localhost"),
        Public(p) => {
            log::debug!("Destination mac: `{}`", p);
            p.parse::<MacAddr6>()
                .expect("failed to parse destination platform as mac address")
        }
    };
    mac.into_array()
}
