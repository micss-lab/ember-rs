use alloc::borrow::Cow;
use core::marker::PhantomData;
use ember_collections::SmallSet;

#[cfg(feature = "acc")]
use ember_acc::Channels;

use ember_core::agent::Agent;
use ember_core::environment::{Environment, SendCallbacks};
use ember_core::message::Payload;
use ember_core::message::TransportMessage;

use crate::adt::{Adt, AgentReference, LocalAgentReference};

/// Message transport service. Delivers messages to agents, manages external message channels and
/// resolves proxied message destinations.
#[derive(Default)]
pub(super) struct Mts<'c> {
    #[cfg(feature = "acc")]
    pub(super) channels: Channels<'c>,
    _lifetime: PhantomData<&'c ()>,
}

impl Agent for Mts<'_> {
    fn get_name(&self) -> Cow<'_, str> {
        Cow::Borrowed("mts")
    }

    fn update(&mut self, _environment: &mut Environment) -> bool {
        // All behaviour of the mts lives at the priviledged level.
        unreachable!("this function should never be called.")
    }
}

impl Mts<'_> {
    pub(super) fn route(
        &mut self,
        message: TransportMessage,
        callbacks: SendCallbacks,
        adt: &mut Adt,
        environment: &mut Environment,
    ) {
        let envelope = &message.envelopes.base;
        // A message can be sent to multiple receivers at once, though callbacks cannot cloned. We'd
        // have to get rid of the `Box<dyn>` around callbacks to avoid this.
        let mut callbacks = Some(callbacks);
        if envelope.to.is_empty() {
            log::error!("Cannot send a message with no receivers");
        } else {
            for t in envelope.to.iter() {
                // Resolve any possible proxies. Error on looping proxies.
                let mut visited = SmallSet::new();
                let mut resolved = t;

                let is_local = loop {
                    if !resolved.is_local() {
                        break false;
                    }

                    match adt.get(resolved.local_name()) {
                        Some(AgentReference::Local(_)) => {
                            break true;
                        }
                        Some(AgentReference::Proxy(proxy)) => {
                            if !visited.insert(proxy.clone()) {
                                log::error!("Proxy loop detected. Message cannot be sent.");
                                return;
                            }
                            resolved = proxy;
                        }
                        None => {
                            if resolved != t {
                                log::error!(
                                    "Failed to send message to agent `{t}` resolved to `{resolved}`: local agent not registered with the ams"
                                );
                            } else {
                                log::error!(
                                    "Failed to send message to agent `{t}`: local agent not registered with the ams"
                                );
                            }

                            if let Some(mut cbs) = callbacks.take() {
                                if let Some(on_failure) = cbs.on_failure.take() {
                                    on_failure(environment);
                                }
                                if let Some(on_complete) = cbs.on_complete.take() {
                                    on_complete(environment);
                                }
                            }
                            return;
                        }
                    }
                };

                drop(visited);

                if is_local {
                    // TODO: Also make use of the callbacks.
                    let local_name = resolved.to_local();
                    let Some(AgentReference::Local(LocalAgentReference { inbox })) =
                        adt.get_mut(local_name.local_name())
                    else {
                        unreachable!("agent is confirmed to be local and to have an inbox above");
                    };
                    if let Payload::AclMessage(message) = message.payload.clone() {
                        inbox.push(message);
                        if let Some(mut cbs) = callbacks.take() {
                            if let Some(on_success) = cbs.on_success.take() {
                                on_success(environment);
                            }
                            if let Some(on_complete) = cbs.on_complete.take() {
                                on_complete(environment);
                            }
                        }
                    } else {
                        // TODO: Solve this.
                        log::warn!("Cannot send message that is not a parsed acl message to agent");
                    }
                } else {
                    #[cfg(feature = "acc")]
                    {
                        use ember_acc::Acc;
                        if self
                            .channels
                            .send(
                                resolved,
                                message.clone(),
                                callbacks.take().unwrap_or_default(),
                                &mut *environment,
                            )
                            .is_ok()
                        {
                            continue;
                        }
                    }

                    if resolved != t {
                        log::error!(
                            "Failed to send message to agent `{t}`, resolved to `{resolved}`."
                        );
                    } else {
                        log::error!("Failed to send message to agent `{t}`.");
                    }
                }
            }
        }
    }
}

#[cfg(feature = "acc")]
impl<'c> Mts<'c> {
    pub(super) fn with_channels(&mut self, channels: Channels<'c>) -> &mut Self {
        self.channels = channels;
        self
    }
}
