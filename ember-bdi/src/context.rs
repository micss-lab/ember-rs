use alloc::vec::Vec;

use crate::event::EventSource;
use crate::intention::IntentionId;
use crate::plan::{Action, TriggeringEvent};

pub struct Context<A> {
    pub(crate) actions: Vec<Action<A>>,
    pub(crate) events: Vec<(EventSource, TriggeringEvent)>,
}

impl<A> Context<A> {
    pub(crate) fn new() -> Self {
        Self {
            actions: Vec::new(),
            events: Vec::new(),
        }
    }

    pub(crate) fn perform_action(&mut self, action: Action<A>) {
        self.actions.push(action);
    }

    pub(crate) fn emit_event(&mut self, event: TriggeringEvent, intention_id: Option<IntentionId>) {
        self.events.push((
            match intention_id {
                Some(id) => EventSource::Internal(id),
                None => EventSource::External,
            },
            event,
        ));
    }
}
