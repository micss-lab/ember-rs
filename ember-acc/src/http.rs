use std::format;
use std::string::{String, ToString};
use std::time::Duration;
use std::vec::Vec;

use bytes::{BufMut, Bytes, BytesMut};
use multipart::server::{Multipart, ReadEntry, ReadEntryResult};
use tiny_http::Server;

use ember_core::agent::aid::Aid;
use ember_core::environment::Environment;
use ember_core::message::{AclRepresentation, Message, MessageEnvelope, Payload};
use ember_core::message::{MessageEnvelopes, TransportMessage, repr};

use super::{Acc, SendError};

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
    fn send(
        &mut self,
        address: &Aid,
        message: TransportMessage,
        mut callbacks: super::SendCallbacks,
        environment: &mut Environment,
    ) -> Result<(), SendError> {
        let envelope = HttpEnvelopeSer {
            to: &message.envelopes.base.to,
            from: &message.envelopes.base.from,
        };
        let envelope_bytes =
            serde_bencode::to_bytes(&envelope).expect("failed to serialize message to bencode");

        let content = match &message.payload {
            Payload::AclMessage(m) => m.to_string(),
            Payload::Bytes(_) => unimplemented!(),
        };

        let mut body = MultipartBody::new();
        body.part("application/custom.mts.env.rep.bencode", &envelope_bytes);
        // TODO: Match the correct payload encoding here.
        body.part(
            "application/fipa.acl.rep.string.std; charset=US-ASCII",
            content.as_bytes(),
        );
        let content_type = format!(
            "multipart/mixed; boundary=\"{}\"; charset=\"ascii\"",
            body.boundary()
        );
        let body = body.finish();

        let result = ureq::post(aid_to_url(address))
            .version(ureq::http::Version::HTTP_11)
            .content_type(content_type)
            .header("HOST", self.server.server_addr().to_string())
            .header("Cache-Control", "no-cache")
            .header("MIME-Version", "1.0")
            .config()
            .timeout_recv_response(Some(Duration::from_millis(50)))
            .build()
            .send(body.as_ref());

        let outcome = match result {
            Ok(response) => {
                log::debug!("Received response: {:?}", response);
                Ok(())
            }
            Err(ureq::Error::Timeout(ureq::Timeout::RecvResponse)) => {
                Err(SendError::Generic("remote acc did not respond".into()))
            }
            Err(e) => Err(SendError::Generic(
                format!("failed to send message: {e}").into(),
            )),
        };

        match outcome.is_ok() {
            true => {
                if let Some(on_success) = callbacks.on_success.take() {
                    on_success(environment);
                }
            }
            false => {
                if let Some(on_failure) = callbacks.on_failure.take() {
                    on_failure(environment);
                }
            }
        }
        if let Some(on_complete) = callbacks.on_complete.take() {
            on_complete(environment);
        }

        outcome
    }

    fn receive(&mut self, _environment: &mut super::Environment) -> Option<TransportMessage> {
        let mut req = self.server.try_recv().expect("receiving message failed")?;
        log::debug!("Request received: {:?}", req);

        let Ok(mut req) = Multipart::from_request(&mut req) else {
            log::error!("Request is not multipart");
            return None;
        };

        let mut buf = Vec::with_capacity(128);

        read_part(&mut req, &mut buf, "message envelope")?;
        log::debug!("Envelope: `{}`", bstr::BString::from(buf.trim_ascii()));
        let envelope = match serde_bencode::from_bytes::<HttpEnvelopeDe>(&buf) {
            Ok(envelope) => envelope,
            Err(e) => {
                log::error!("Error parsing message envelope: {}", e);
                return None;
            }
        };

        read_part(&mut req, &mut buf, "message")?;
        log::debug!("Acl message: `{}`", bstr::BString::from(buf.trim_ascii()));
        let message = match repr::payload::string::decode(buf.as_slice()) {
            Ok(message) => message,
            Err(_e) => {
                log::error!("Error parsing acl message");
                return None;
            }
        };

        Some(envelope.with_content(message))
    }
}

fn read_part<M: ReadEntry>(multipart: &mut M, buf: &mut Vec<u8>, label: &str) -> Option<()> {
    use std::io::Read;

    let ReadEntryResult::Entry(mut entry) = multipart.read_entry_mut() else {
        log::error!("Error extracting {label} from multipart request");
        return None;
    };

    buf.clear();
    let len = entry
        .data
        .read_to_end(buf)
        .expect("failed to read multipart entry data");
    log::trace!("Read {label} of length {len} bytes");
    Some(())
}

#[derive(serde::Serialize)]
struct HttpEnvelopeSer<'a> {
    to: &'a [Aid],
    from: &'a Aid,
}

#[derive(serde::Deserialize)]
struct HttpEnvelopeDe {
    to: Vec<Aid>,
    from: Aid,
}

impl HttpEnvelopeDe {
    fn with_content(self, message: Message) -> TransportMessage {
        let Self { to, from } = self;
        let envelope = MessageEnvelope {
            to,
            from,
            date: chrono::DateTime::<chrono::Utc>::MIN_UTC.into(),
            acl_representation: AclRepresentation::String,
            other: None,
        };
        TransportMessage {
            envelopes: MessageEnvelopes {
                base: envelope,
                others: Vec::with_capacity(0),
            },
            payload: Payload::AclMessage(message),
        }
    }
}

struct MultipartBody {
    body: BytesMut,
    boundary: String,
}

impl MultipartBody {
    fn new() -> Self {
        use rand::RngCore;
        let mut boundary = [0u8; 16];
        rand::rng().fill_bytes(&mut boundary);

        let mut body = BytesMut::new();
        body.put_slice(b"This is not part of the MIME multipart encoded message.\r\n");
        Self {
            body,
            boundary: hex::encode(boundary),
        }
    }

    fn boundary(&self) -> &str {
        &self.boundary
    }

    fn part(&mut self, content_type: &str, data: &[u8]) -> &mut Self {
        self.body.put_slice(b"--");
        self.body.put_slice(self.boundary.as_bytes());
        self.body.put_slice(b"\r\n");
        self.body.put_slice(b"Content-Type: ");
        self.body.put_slice(content_type.as_bytes());
        self.body.put_slice(b"\r\n\r\n");
        self.body.put_slice(data);
        self.body.put_slice(b"\r\n\r\n");
        self
    }

    fn finish(mut self) -> Bytes {
        self.body.put_slice(b"--");
        self.body.put_slice(self.boundary.as_bytes());
        self.body.put_slice(b"--\r\n\r\n");
        self.body.freeze()
    }
}

fn aid_to_url(aid: &Aid) -> String {
    use ember_core::agent::aid::AgentPlatform::*;
    let host = match aid.platform() {
        Local => "localhost",
        Public(h) => h,
    };
    format!("http://{}/acc", host)
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    #[test]
    fn envelope_bencode_round_trip() {
        let from = Aid::local("alice");
        let to = vec![Aid::local("bob")];
        let ser = HttpEnvelopeSer {
            to: &to,
            from: &from,
        };
        let bytes = serde_bencode::to_bytes(&ser).expect("serialize");
        let de: HttpEnvelopeDe = serde_bencode::from_bytes(&bytes).expect("deserialize");
        assert_eq!(de.to, to);
        assert_eq!(de.from, from);
    }
}
