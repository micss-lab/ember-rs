use alloc::vec::Vec;
use derive_where::derive_where;

use ember_core::environment::Environment;

use crate::behaviour::BehaviourId;

pub struct Context<'ctx, E> {
    pub(crate) environment: &'ctx mut Environment,
    pub(crate) agent: &'ctx mut AgentContext,
    pub(crate) local: LocalContext<E>,
}

impl<E> Context<'_, E> {
    pub fn emit_event(&mut self, event: E) {
        self.local.events.push(event);
    }

    pub fn remove_agent(&mut self) {
        self.agent.should_remove = true;
    }

    pub fn block_behaviour(&mut self) {
        self.local.block = true;
    }

    pub fn reset_behaviour(&mut self) {
        self.local.reset = true;
    }

    pub fn remove_behaviour(&mut self, id: BehaviourId) {
        self.local.removed_behaviours.push(id);
    }
}

impl<'ctx, E> Context<'ctx, E> {
    pub(crate) fn new(environment: &'ctx mut Environment, agent: &'ctx mut AgentContext) -> Self {
        Self {
            environment,
            agent,
            local: LocalContext::default(),
        }
    }

    pub(crate) fn fresh_local<'a, CE>(&'a mut self) -> Context<'a, CE>
    where
        'ctx: 'a,
    {
        Context {
            environment: self.environment,
            agent: self.agent,
            local: LocalContext::default(),
        }
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

#[derive(Default)]
pub(crate) struct AgentContext {
    pub(crate) should_remove: bool,
}

#[derive_where(Default)]
pub(crate) struct LocalContext<E> {
    pub(crate) events: Vec<E>,
    pub(crate) removed_behaviours: Vec<BehaviourId>,
    pub(crate) block: bool,
    pub(crate) reset: bool,
}
