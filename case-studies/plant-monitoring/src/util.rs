use ember::message::{Message, MessageEnvelope, Receiver};

pub fn wrap_message(m: Message) -> MessageEnvelope {
    let Receiver::Single(ref r) = m.receiver else {
        unimplemented!();
    };
    MessageEnvelope::new(r.clone(), m)
}
