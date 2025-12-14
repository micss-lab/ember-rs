#[cfg(feature = "acc-custom")]
use alloc::boxed::Box;
use alloc::collections::BTreeSet;

#[cfg(feature = "acc-espnow")]
use esp_wifi::esp_now;

use ember_acc::{Acc, Channels};
use ember_core::message::MessageEnvelope;

use crate::adt::{Adt, AgentReference, LocalAgentReference};

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
                // Resolve any possible proxies. Error on looping proxies.
                let (mut resolved, mut visited) = (t.clone(), BTreeSet::new());

                if let Some(inbox) = loop {
                    if !resolved.is_local() {
                        break None;
                    }

                    match adt.get_mut(resolved.local_name()) {
                        Some(AgentReference::Local(LocalAgentReference { inbox })) => {
                            break Some(inbox);
                        }
                        Some(AgentReference::Proxy(proxy)) => {
                            if !visited.insert(proxy.clone()) {
                                log::error!("Proxy loop detected. Message cannot be sent.");
                                return;
                            }
                            resolved = proxy.clone();
                        }
                        None => {
                            log::error!(
                                "Failed to send message to agent `{t}`: local agent not registered with the ams"
                            );
                            return;
                        }
                    }
                } {
                    inbox.push(envelope.clone());
                } else if self.channels.send(&resolved, envelope.clone()).is_err() {
                    log::error!("Failed to send message to agent `{t}`.");
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

impl<'c> Mts<'c> {
    #[cfg(feature = "acc-http")]
    pub(super) fn enable_http(&mut self, port: u16) {
        self.channels.enable_http(port);
    }

    #[cfg(feature = "acc-espnow")]
    pub(super) fn enable_espnow(
        &mut self,
        sender: Option<esp_now::EspNowSender<'c>>,
        receiver: Option<esp_now::EspNowReceiver<'c>>,
    ) {
        self.channels.enable_espnow(sender, receiver);
    }

    #[cfg(feature = "acc-custom")]
    pub(super) fn enable_custom_acc(&mut self, custom: Box<dyn Acc + 'c>) {
        self.channels.enable_custom(custom);
    }
}
