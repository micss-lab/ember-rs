use alloc::boxed::Box;

use ember_core::{
    acl::message::{Message, MessageEnvelope, Receiver},
    behaviour::{Behaviour, BehaviourId, IntoBehaviour},
};

pub fn wrap_message(m: Message) -> MessageEnvelope {
    let Receiver::Single(ref r) = m.receiver else {
        unimplemented!();
    };
    MessageEnvelope::new(r.clone(), m)
}

pub fn behaviour_with_id<K, A: 'static, E: 'static>(
    behaviour: impl IntoBehaviour<K, AgentState = A, Event = E>,
) -> (BehaviourId, Box<dyn Behaviour<AgentState = A, Event = E>>) {
    let behaviour = behaviour.into_behaviour();
    (behaviour.id(), behaviour)
}
