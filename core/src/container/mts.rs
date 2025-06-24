#[cfg(feature = "acc-espnow")]
use esp_wifi::esp_now;

use crate::acc::{Acc, Channels};
use crate::acl::message::MessageEnvelope;
use crate::adt::Adt;

pub(super) struct Mts<'c> {
    channels: Channels<'c>,
}

impl Mts<'_> {
    pub(super) fn new() -> Self {
        Mts {
            channels: Channels::new(),
        }
    }

    pub(super) fn send_message(&mut self, envelope: MessageEnvelope, adt: &mut Adt) {
        if envelope.to.is_empty() {
            log::error!("Cannot send a message with no receivers");
        } else {
            for t in envelope.to.iter() {
                if t.is_local() {
                    match adt.get_mut(&t) {
                        Some(i) => i.inbox.push(envelope.clone()),
                        None => {
                            log::error!(
                                "Failed to send message to agent `{}`: local agent not registered with the ams",
                                t
                            );
                        }
                    }
                } else {
                    // TODO: Handle/print the error.
                    if let Err(_) = self.channels.send(t, envelope.clone()) {
                        log::error!("Failed to send message to agent `{}`.", t);
                    }
                }
            }
        }
    }

    pub(super) fn receive_messages(&mut self, adt: &mut Adt) {
        while let Some(mut message) = self.channels.receive() {
            // TODO: Do this according to the fipa spec.
            // Set the to parameter to the local address of the agent.
            message.to = message.to.into_iter().map(|t| t.to_local()).collect();

            // Send the message as if it was to the local agent.
            self.send_message(message, &mut *adt);
        }
    }
}

impl Mts<'_> {
    #[cfg(feature = "acc-http")]
    pub(super) fn enable_http(&mut self, port: u16) {
        self.channels.enable_http(port);
    }
}

impl<'c> Mts<'c> {
    #[cfg(feature = "acc-espnow")]
    pub(super) fn enable_espnow(
        &mut self,
        sender: Option<esp_now::EspNowSender<'c>>,
        receiver: Option<esp_now::EspNowReceiver<'c>>,
    ) {
        self.channels.enable_espnow(sender, receiver)
    }
}
