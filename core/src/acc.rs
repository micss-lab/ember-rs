use crate::{acl::message::MessageEnvelope, Aid};

use self::http::HttpChannel;

mod http;

pub(crate) trait Acc {
    fn send(&mut self, aid: &Aid, message: MessageEnvelope) -> Result<(), ()>;

    fn receive(&mut self) -> Option<MessageEnvelope>;
}

pub(crate) struct Channels {
    http: Option<HttpChannel>,
}

impl Channels {
    pub(crate) fn new() -> Self {
        Self { http: None }
    }

    pub(crate) fn enable_http(&mut self, port: u16) {
        if self.http.is_some() {
            log::warn!("Http already enabled. Nothing changed.");
            return;
        }
        self.http = Some(HttpChannel::new(port))
    }
}

impl Acc for Channels {
    fn send(&mut self, address: &Aid, message: MessageEnvelope) -> Result<(), ()> {
        self.http
            .as_mut()
            .map_or(Err(()), |http| http.send(address, message))
    }

    fn receive(&mut self) -> Option<MessageEnvelope> {
        self.http.as_mut()?.receive()
    }
}
