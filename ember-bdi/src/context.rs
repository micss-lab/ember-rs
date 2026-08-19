use alloc::borrow::Cow;
use alloc::rc::Rc;
use alloc::vec::Vec;

use ember_core::environment::Environment;

use crate::event::EventSource;
use crate::intention::IntentionId;
use crate::plan::TriggeringEvent;
use crate::plan::action::PendingAction;

/// What a pure built-in action (`.now`, `.me`) needs to evaluate, whether that's inside a
/// plan body or a context guard. Kept separate from `Context` so that a future pure action
/// needing something new grows this struct instead of another bare parameter threaded
/// through the whole query engine.
#[derive(Debug, Clone)]
pub struct PureContext {
    pub(crate) agent_name: Rc<Cow<'static, str>>,
}

impl PureContext {
    pub(crate) fn new(agent_name: Rc<Cow<'static, str>>) -> Self {
        Self { agent_name }
    }
}

pub struct Context<'ctx, A> {
    pub(crate) pure: PureContext,
    pub(crate) actions: Vec<(Option<IntentionId>, PendingAction<A>)>,
    pub(crate) events: Vec<(EventSource, TriggeringEvent)>,
    pub(crate) environment: &'ctx mut Environment,
}

impl<'ctx, A> Context<'ctx, A> {
    pub(crate) fn new(pure: PureContext, environment: &'ctx mut Environment) -> Self {
        Self {
            pure,
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
