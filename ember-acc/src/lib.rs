#![no_std]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

use core::marker::PhantomData;

use ember_core::agent::aid::Aid;
use ember_core::message::MessageEnvelope;

#[cfg(feature = "espnow")]
use self::espnow::*;
#[cfg(feature = "http")]
use self::http::*;

#[cfg(feature = "espnow")]
mod espnow;
#[cfg(feature = "http")]
mod http;

pub trait Acc {
    fn send(&mut self, aid: &Aid, message: MessageEnvelope) -> Result<(), ()>;

    fn receive(&mut self) -> Option<MessageEnvelope>;
}

#[derive(Default)]
pub struct Channels<'c> {
    #[cfg(feature = "http")]
    http: Option<HttpChannel>,
    #[cfg(feature = "espnow")]
    espnow: Option<EspNowChannel<'c>>,
    _lifetime: PhantomData<&'c ()>,
}

impl Channels<'_> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Channels<'_> {
    #[cfg(feature = "http")]
    pub fn enable_http(&mut self, port: u16) {
        if self.http.is_some() {
            log::warn!("Http already enabled. Nothing changed.");
            return;
        }
        self.http = Some(HttpChannel::new(port));
    }
}

impl<'c> Channels<'c> {
    #[cfg(feature = "espnow")]
    pub fn enable_espnow(
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
            if #[cfg(feature = "http")] {
                self.http
                    .as_mut()
                    .map_or(Err(()), |http| http.send(address, message))
            } else if #[cfg(feature = "espnow")] {
                self.espnow.as_mut().map_or(Err(()), |espnow| espnow.send(address, message))
            } else {
                let _ = (address, message);
                Ok(())
            }
        }
    }

    fn receive(&mut self) -> Option<MessageEnvelope> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "http")] {
                self.http.as_mut()?.receive()
            } else if #[cfg(feature = "espnow")] {
                self.espnow.as_mut()?.receive()
            } else {
                None
            }
        }
    }
}
