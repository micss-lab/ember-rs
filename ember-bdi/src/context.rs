use alloc::vec::Vec;

use crate::event::EventSource;
use crate::intention::IntentionId;
use crate::plan::TriggeringEvent;

pub struct Context<A> {
    actions: Vec<A>,
    events: Vec<(EventSource, TriggeringEvent)>,
}

impl<A> Context<A> {
    pub(crate) fn perform_action(&mut self, action: A) {
        self.actions.push(action);
    }

    pub(crate) fn emit_event(&mut self, event: TriggeringEvent, intention_id: IntentionId) {
        self.events
            .push((EventSource::Internal(intention_id), event));
    }
}
