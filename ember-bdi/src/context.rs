use alloc::vec::Vec;

use ember_core::environment::Environment;

use crate::event::EventSource;
use crate::intention::IntentionId;
use crate::plan::{Action, TriggeringEvent};

pub struct Context<'ctx, A> {
    pub(crate) actions: Vec<Action<A>>,
    pub(crate) events: Vec<(EventSource, TriggeringEvent)>,
    pub(crate) environment: &'ctx mut Environment,
}

impl<'ctx, A> Context<'ctx, A> {
    pub(crate) fn new(environment: &'ctx mut Environment) -> Self {
        Self {
            actions: Vec::new(),
            events: Vec::new(),
            environment,
        }
    }
}

impl<A> Context<'_, A> {
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

impl<E> core::ops::Deref for Context<'_, E> {
    type Target = Environment;

    fn deref(&self) -> &Self::Target {
        self.environment
    }
}

impl<E> core::ops::DerefMut for Context<'_, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.environment
    }
}
