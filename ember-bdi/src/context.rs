use alloc::vec::Vec;

use crate::plan::TriggeringEvent;

pub struct Context<A> {
    actions: Vec<A>,
    events: Vec<TriggeringEvent>,
}

impl<A> Context<A> {
    pub(crate) fn perform_action(&mut self, action: A) {
        self.actions.push(action);
    }

    pub(crate) fn emit_event(&mut self, event: TriggeringEvent) {
        self.events.push(event);
    }
}
