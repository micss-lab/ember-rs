use crate::{acl::message::MessageEnvelope, Aid};

#[cfg(not(target_os = "none"))]
use self::http::HttpChannel;

#[cfg(not(target_os = "none"))]
mod http;

pub(crate) trait Acc {
    fn send(&mut self, aid: &Aid, message: MessageEnvelope) -> Result<(), ()>;

    fn receive(&mut self) -> Option<MessageEnvelope>;
}

pub(crate) struct Channels {
    #[cfg(not(target_os = "none"))]
    http: Option<HttpChannel>,
}

impl Channels {
    pub(crate) fn new() -> Self {
        Self {
            #[cfg(not(target_os = "none"))]
            http: None,
        }
    }

    #[cfg(not(target_os = "none"))]
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
        cfg_if::cfg_if! {
            if #[cfg(not(target_os = "none"))] {
                self.http
                    .as_mut()
                    .map_or(Err(()), |http| http.send(address, message))
            } else {
                Ok(())
            }
        }
    }

    fn receive(&mut self) -> Option<MessageEnvelope> {
        cfg_if::cfg_if! {
            if #[cfg(not(target_os = "none"))] {
                self.http.as_mut()?.receive()
            } else {
                None
            }
        }
    }
}
