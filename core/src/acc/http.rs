extern crate std;

use std::collections::BTreeMap;
use std::format;
use std::string::{String, ToString};
use std::time::Duration;
use std::vec::Vec;

use bytes::{BufMut, Bytes, BytesMut};
use multipart::server::{Multipart, ReadEntry, ReadEntryResult};
use serde::ser::SerializeStruct;
use tiny_http::Server;

use crate::acl::message::{AclRepresentation, Message, MessageEnvelope, MessageKind};
use crate::agent::Aid;

use super::Acc;

pub(super) struct HttpChannel {
    server: Server,
}

impl HttpChannel {
    pub(super) fn new(port: u16) -> Self {
        Self {
            server: tiny_http::Server::http(format!("0.0.0.0:{}", port)).unwrap(),
        }
    }
}

impl Acc for HttpChannel {
    fn send(&mut self, address: &Aid, message: MessageEnvelope) -> Result<(), ()> {
        use rand::RngCore;
        let mut boundary = [0u8; 16];
        rand::rng().fill_bytes(&mut boundary);

        let response = match ureq::post(aid_to_url(address))
            .version(ureq::http::Version::HTTP_11)
            .content_type(format!(
                "multipart/mixed; boundary=\"{}\"; charset=\"ascii\"",
                hex::encode(boundary)
            ))
            .header("HOST", self.server.server_addr().to_string())
            .header("Cache-Control", "no-cache")
            .header("MIME-Version", "1.0")
            .config()
            .timeout_recv_response(Some(Duration::from_millis(50)))
            .build()
            .send(encode_message(message, &boundary).as_ref())
        {
            Ok(res) => res,
            Err(ureq::Error::Timeout(ureq::Timeout::RecvResponse)) => {
                // TODO: Handle this.
                log::warn!("Remote acc did not respond");
                return Ok(());
            }
            Err(e) => Err(e).expect("failed to send message"),
        };

        log::debug!("Received response: {:?}", response);

        Ok(())
    }

    fn receive(&mut self) -> Option<MessageEnvelope> {
        use std::io::Read;

        let mut req = self.server.try_recv().expect("receiving message failed")?;
        log::debug!("Request received: {:?}", req);

        let Ok(mut req) = Multipart::from_request(&mut req) else {
            log::error!("Request is not multipart");
            return None;
        };

        let ReadEntryResult::Entry(mut envelope) = req.read_entry_mut() else {
            log::error!("Error extracting message envelope from multipart request");
            return None;
        };

        let mut buf = Vec::with_capacity(128);
        let len = envelope
            .data
            .read_to_end(&mut buf)
            .expect("failed to read envelope data");
        log::trace!("Read envelope of length {} bytes", len);
        log::debug!("Envelope: `{}`", bstr::BString::from(buf.trim_ascii()));

        let envelope = match serde_bencode::from_bytes::<HttpEnvelopeDe>(&buf) {
            Ok(envelope) => envelope,
            Err(e) => {
                log::error!("Error parsing message envelope: {}", e);
                return None;
            }
        };

        let ReadEntryResult::Entry(mut message) = req.read_entry_mut() else {
            log::error!("Error extracting message from multipart request");
            return None;
        };

        buf.clear();
        let len = message
            .data
            .read_to_end(&mut buf)
            .expect("failed to read acl message data");
        log::trace!("Read acl message of length {} bytes", len);
        log::debug!("Acl message: `{}`", bstr::BString::from(buf.trim_ascii()));

        let message = match Message::try_from_bytes(buf.as_slice()) {
            Ok(message) => message,
            Err(_e) => {
                log::error!("Error parsing acl message");
                return None;
            }
        };

        Some(envelope.with_content(message))
    }
}

struct HttpEnvelopeSer<'a>(&'a MessageEnvelope);
struct HttpEnvelopeDe {
    to: Vec<Aid>,
    from: Option<Aid>,
}

impl serde::Serialize for HttpEnvelopeSer<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // TODO: Serialize other parameters as well.
        let mut envelope = serializer.serialize_struct("envelope", 2)?;
        envelope.serialize_field("to", &self.0.to)?;
        envelope.serialize_field("from", &self.0.from)?;
        envelope.end()
    }
}

impl<'de> serde::Deserialize<'de> for HttpEnvelopeDe {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        enum Field {
            To,
            From,
        }

        impl<'de> serde::Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> serde::de::Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str("`to` or `from`")
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        Ok(match v {
                            "to" => Field::To,
                            "from" => Field::From,
                            _ => return Err(serde::de::Error::unknown_field(v, FIELDS)),
                        })
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct HttpEnvelopeDeVisitor;

        impl<'de> serde::de::Visitor<'de> for HttpEnvelopeDeVisitor {
            type Value = HttpEnvelopeDe;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct envelope")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut to = None;
                let mut from = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::To => {
                            if to.is_some() {
                                return Err(serde::de::Error::duplicate_field("to"));
                            }
                            to = Some(map.next_value()?);
                        }
                        Field::From => {
                            if from.is_some() {
                                return Err(serde::de::Error::duplicate_field("from"));
                            }
                            from = Some(map.next_value()?);
                        }
                    }
                }
                let to = to.ok_or_else(|| serde::de::Error::missing_field("to"))?;
                Ok(HttpEnvelopeDe { to, from })
            }
        }

        const FIELDS: &[&str] = &["to", "from"];
        deserializer.deserialize_struct("envelope", FIELDS, HttpEnvelopeDeVisitor)
    }
}

impl HttpEnvelopeDe {
    fn with_content(self, message: Message) -> MessageEnvelope {
        let Self { to, from } = self;
        MessageEnvelope {
            to,
            from,
            date: chrono::DateTime::<chrono::Utc>::MIN_UTC.into(),
            acl_representation: AclRepresentation::BitEfficient,
            parameters: BTreeMap::new(),
            message: MessageKind::Structured(message),
        }
    }
}

fn encode_message(message: MessageEnvelope, boundary: &[u8; 16]) -> Bytes {
    let boundary = hex::encode(boundary);
    let mut body = BytesMut::new();

    // Preamble.
    body.put_slice(b"This is not part of the MIME multipart encoded message.");
    body.put_slice(b"\r\n");

    // Message Envelope boundary.
    body.put_slice(b"--");
    body.put_slice(boundary.as_bytes());
    body.put_slice(b"\r\n");

    // Message Envelope headers.
    body.put_slice(b"Content-Type: application/custom.mts.env.rep.bencode");
    body.put_slice(b"\r\n");
    body.put_slice(b"\r\n");

    // Message Envelope.
    body.put(
        &*serde_bencode::to_bytes(&HttpEnvelopeSer(&message))
            .expect("failed to serialize message to bencode"),
    );
    body.put_slice(b"\r\n");
    body.put_slice(b"\r\n");

    // Message Body boundary.
    body.put_slice(b"--");
    body.put_slice(boundary.as_bytes());
    body.put_slice(b"\r\n");

    // Message Body headers.
    // TODO: Match the correct payload encoding here.
    body.put_slice(b"Content-Type: application/fipa.acl.rep.string.std; charset=US-ASCII");
    body.put_slice(b"\r\n");
    body.put_slice(b"\r\n");

    // Message Body.
    match message.message {
        MessageKind::Structured(m) => body.put_slice(m.to_string().as_bytes()),
    }
    body.put_slice(b"\r\n");
    body.put_slice(b"\r\n");

    // End boundary.
    body.put_slice(b"--");
    body.put_slice(boundary.as_bytes());
    body.put_slice(b"--");
    body.put_slice(b"\r\n");
    body.put_slice(b"\r\n");

    body.freeze()
}

fn aid_to_url(aid: &Aid) -> String {
    use crate::agent::AgentPlatform::*;
    let host = match aid.platform() {
        Local => "localhost",
        Public(h) => h,
    };
    format!("http://{}/acc", host)
}
