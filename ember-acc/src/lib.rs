#![no_std]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "custom")]
use alloc::boxed::Box;
use core::marker::PhantomData;

use ember_core::agent::aid::Aid;
use ember_core::message::TransportMessage;

pub use ember_core::environment::{Environment, SendCallbacks};

#[cfg(feature = "espnow")]
use self::espnow::*;
#[cfg(feature = "http")]
use self::http::*;

#[cfg(feature = "espnow")]
mod espnow;
#[cfg(feature = "http")]
mod http;

#[cfg(feature = "espnow")]
pub use self::espnow::ReliableEspNowChannel;

#[cfg(any(feature = "serde-espnow", feature = "serde-http"))]
pub mod serde;

#[cfg(feature = "espnow")]
pub mod util {
    use macaddr::MacAddr6;

    use ember_core::agent::aid::Aid;

    pub fn aid_to_mac(aid: &Aid) -> [u8; 6] {
        use ember_core::agent::aid::AgentPlatform::*;
        let mac = match aid.platform() {
            Local => panic!("espnow channel does not support sending messages to localhost"),
            Public(p) => p
                .parse::<MacAddr6>()
                .expect("failed to parse destination platform as mac address"),
        };
        mac.into_array()
    }
}

pub trait Acc {
    fn send(
        &mut self,
        aid: &Aid,
        message: TransportMessage,
        callbacks: SendCallbacks,
    ) -> Result<(), ()>;

    fn receive(&mut self, environment: &mut Environment) -> Option<TransportMessage>;
}

#[derive(Default)]
pub struct Channels<'c> {
    #[cfg(feature = "http")]
    http: Option<HttpChannel>,
    #[cfg(feature = "espnow")]
    espnow: Option<EspNowChannel<'c>>,
    #[cfg(feature = "custom")]
    custom: Option<Box<dyn Acc + 'c>>,

    _lifetime: PhantomData<&'c ()>,
}

impl Channels<'_> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<'c> Channels<'c> {
    #[cfg(feature = "http")]
    pub fn enable_http(&mut self, port: u16) {
        if self.http.is_some() {
            log::warn!("Http already enabled. Nothing changed.");
            return;
        }
        self.http = Some(HttpChannel::new(port));
    }

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

    #[cfg(feature = "custom")]
    pub fn enable_custom(&mut self, custom: Box<dyn Acc + 'c>) {
        if self.custom.is_some() {
            log::warn!("Custom access channel already enabled. Nothing changed.");
        }
        self.custom = Some(custom);
    }
}

impl Acc for Channels<'_> {
    fn send(
        &mut self,
        address: &Aid,
        message: TransportMessage,
        callbacks: SendCallbacks,
    ) -> Result<(), ()> {
        #[cfg(feature = "custom")]
        if let Some(custom) = self.custom.as_mut() {
            return custom.send(address, message, callbacks);
        }
        #[cfg(feature = "espnow")]
        if let Some(espnow) = self.espnow.as_mut() {
            return espnow.send(address, message, callbacks);
        }
        #[cfg(feature = "http")]
        if let Some(http) = self.http.as_mut() {
            return http.send(address, message, callbacks);
        }
        let _ = (address, message, callbacks);
        Err(())
    }

    fn receive(&mut self, environment: &mut Environment) -> Option<TransportMessage> {
        #[cfg(feature = "custom")]
        if let Some(custom) = self.custom.as_mut() {
            return custom.receive(environment);
        }
        #[cfg(feature = "espnow")]
        if let Some(espnow) = self.espnow.as_mut() {
            return espnow.receive(environment);
        }
        #[cfg(feature = "http")]
        if let Some(http) = self.http.as_mut() {
            return http.receive(environment);
        }
        let _ = environment;
        None
    }
}
