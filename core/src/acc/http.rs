extern crate std;

use alloc::string::ToString;
use core::time::Duration;
use std::format;

use bytes::{BufMut, Bytes, BytesMut};
use multipart_any::server::{HttpRequest, Multipart};
use serde::ser::SerializeStruct;
use tiny_http::Server;

use crate::{
    acl::message::{MessageEnvelope, MessageKind},
    Aid,
};

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

        let response = match ureq::post(address.to_transport_address())
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
        let req = self.server.try_recv().expect("receiving message failed")?;
        log::debug!("Request received: {:?}", req);

        let Ok(req) = Multipart::from_request(req as HttpRequest) else {
            log::error!("Request is not multipart");
            return None;
        };
        None
    }
}

struct HttpEnvelopeSer<'a>(&'a MessageEnvelope);

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

fn encode_message(message: MessageEnvelope, boundary: &[u8; 16]) -> Bytes {
    let mut body = BytesMut::new();

    // Preamble.
    body.put_slice(b"This is not part of the MIME multipart encoded message.");
    body.put_slice(b"\r\n");

    // Message Envelope boundary.
    body.put_slice(b"--");
    body.put_slice(boundary);
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
    body.put_slice(boundary);
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
    body.put_slice(boundary);
    body.put_slice(b"--");
    body.put_slice(b"\r\n");
    body.put_slice(b"\r\n");

    body.freeze()
}
