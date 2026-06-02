use alloc::borrow::Cow;

use ember_core::agent::Agent;
use ember_core::environment::Environment;
use ember_fipa::agent::{ExecutionState, FipaAgent};

use crate::behaviour::parallel::{FinishStrategy, ParallelBehaviourQueue};
use crate::behaviour::{BehaviourId, IntoBehaviour};
use crate::context::{AgentContext, Context};

pub struct ReactiveAgent<'a, S, E> {
    pub(crate) name: Cow<'static, str>,
    behaviours: ParallelBehaviourQueue<'a, S, E>,
    fipa: FipaAgent,
    state: S,
}

impl<S, E> ReactiveAgent<'_, S, E> {
    pub fn new(name: impl Into<Cow<'static, str>>, state: S) -> Self {
        Self {
            name: name.into(),
            behaviours: ParallelBehaviourQueue::new_empty(FinishStrategy::Never),
            fipa: FipaAgent::default(),
            state,
        }
    }
}

impl<'a, S, E> ReactiveAgent<'a, S, E> {
    pub fn with_behaviour<'b, K>(
        mut self,
        behaviour: impl IntoBehaviour<'b, K, AgentState = S, Event = E>,
    ) -> Self
    where
        'b: 'a,
    {
        self.add_behaviour(behaviour);
        self
    }

    pub fn add_behaviour<'b, K>(
        &mut self,
        behaviour: impl IntoBehaviour<'b, K, AgentState = S, Event = E>,
    ) -> BehaviourId
    where
        'b: 'a,
    {
        let behaviour = behaviour.into_behaviour();
        let id = behaviour.id();
        self.behaviours.add_behaviour(behaviour);
        id
    }
}

impl<S, E> Agent for ReactiveAgent<'_, S, E> {
    fn update(&mut self, environment: &mut Environment) -> bool {
        use crate::behaviour::complex::scheduler::BehaviourScheduler;

        // log::trace!("Ticking agent `{}`", self.name);

        match self.fipa.update(&mut *environment, &self.name) {
            ExecutionState::Initiated => return false,
            ExecutionState::Active => (),
        }

        let mut agent_context = AgentContext::default();
        let mut context = Context::new(environment, &mut agent_context);
        self.behaviours.action(&mut context, &mut self.state);

        false
    }

    fn get_name(&self) -> Cow<str> {
        self.name.clone()
    }
}
