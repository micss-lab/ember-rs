use alloc::borrow::Cow;
use alloc::rc::Rc;
use alloc::vec::Vec;

use ember_core::environment::Environment;

use crate::event::EventSource;
use crate::intention::IntentionId;
use crate::plan::TriggeringEvent;
use crate::plan::action::PendingAction;

pub struct Context<'ctx, A> {
    pub(crate) agent_name: Rc<Cow<'static, str>>,
    pub(crate) actions: Vec<(Option<IntentionId>, PendingAction<A>)>,
    pub(crate) events: Vec<(EventSource, TriggeringEvent)>,
    pub(crate) environment: &'ctx mut Environment,
}

impl<'ctx, A> Context<'ctx, A> {
    pub(crate) fn new(
        agent_name: Rc<Cow<'static, str>>,
        environment: &'ctx mut Environment,
    ) -> Self {
        Self {
            agent_name,
            actions: Vec::new(),
            events: Vec::new(),
            environment,
        }
    }
}

impl<A> Context<'_, A> {
    /// Instead of running an action immediately, the action is pushed to the agent as a pending
    /// action. Setting `intention` blocks the given intention for the time action remains
    /// pending.
    pub(crate) fn dispatch_action(
        &mut self,
        action: PendingAction<A>,
        intention: Option<IntentionId>,
    ) {
        self.actions.push((intention, action));
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

impl<A> core::ops::Deref for Context<'_, A> {
    type Target = Environment;

    fn deref(&self) -> &Self::Target {
        self.environment
    }
}

impl<A> core::ops::DerefMut for Context<'_, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.environment
    }
}
