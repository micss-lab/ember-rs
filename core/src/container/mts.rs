use crate::acl::message::MessageEnvelope;
use crate::adt::Adt;

pub(super) fn send_message(envelope: MessageEnvelope, adt: &mut Adt) {
    // Figure out the agents this message is for.
    let to = envelope.to.clone();
    if to.is_empty() {
        // Message is a broadcast.
        adt.values_mut()
            .for_each(|i| i.inbox.push(envelope.clone()));
    } else {
        to.into_iter().for_each(|t| match adt.get_mut(&t) {
            Some(i) => i.inbox.push(envelope.clone()),
            None => {
                log::error!(
                    "Failed to send message to agent `{}`: not registered with the ams",
                    t
                );
            }
        })
    }
}
