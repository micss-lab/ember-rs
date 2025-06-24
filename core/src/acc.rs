use core::marker::PhantomData;

use crate::{acl::message::MessageEnvelope, Aid};

#[cfg(feature = "acc-espnow")]
use self::espnow::{EspNowChannel, EspNowReceiver, EspNowSender};
#[cfg(feature = "acc-http")]
use self::http::HttpChannel;

#[cfg(feature = "acc-espnow")]
mod espnow;
#[cfg(feature = "acc-http")]
mod http;

pub(crate) trait Acc {
    fn send(&mut self, aid: &Aid, message: MessageEnvelope) -> Result<(), ()>;

    fn receive(&mut self) -> Option<MessageEnvelope>;
}

pub(crate) struct Channels<'c> {
    #[cfg(feature = "acc-http")]
    http: Option<HttpChannel>,
    #[cfg(feature = "acc-espnow")]
    espnow: Option<EspNowChannel<'c>>,
    _lifetime: PhantomData<&'c ()>,
}

impl Channels<'_> {
    pub(crate) fn new() -> Self {
        Self {
            #[cfg(feature = "acc-http")]
            http: None,
            #[cfg(feature = "acc-espnow")]
            espnow: None,
            _lifetime: PhantomData,
        }
    }

    #[cfg(feature = "acc-http")]
    pub(crate) fn enable_http(&mut self, port: u16) {
        if self.http.is_some() {
            log::warn!("Http already enabled. Nothing changed.");
            return;
        }
        self.http = Some(HttpChannel::new(port));
    }
}

impl<'c> Channels<'c> {
    #[cfg(feature = "acc-espnow")]
    pub(crate) fn enable_espnow(
        &mut self,
        sender: Option<EspNowSender<'c>>,
        receiver: Option<EspNowReceiver<'c>>,
    ) {
        if self.espnow.is_some() {
            log::warn!("EspNow already enabled. Nothing changed.");
            return;
        }
        self.espnow = Some(EspNowChannel::new(sender, receiver));
    }
}

impl Acc for Channels<'_> {
    fn send(&mut self, address: &Aid, message: MessageEnvelope) -> Result<(), ()> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "acc-http")] {
                self.http
                    .as_mut()
                    .map_or(Err(()), |http| http.send(address, message))
            } else if #[cfg(feature = "acc-espnow")] {
                self.espnow.as_mut().map_or(Err(()), |espnow| espnow.send(address, message))
            } else {
                Ok(())
            }
        }
    }

    fn receive(&mut self) -> Option<MessageEnvelope> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "acc-http")] {
                self.http.as_mut()?.receive()
            } else if #[cfg(feature = "acc-espnow")] {
                self.espnow.as_mut()?.receive()
            } else {
                None
            }
        }
    }
}
