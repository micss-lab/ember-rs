use alloc::borrow::Cow;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use uchan::{Receiver, Sender};

use crate::acl::message::{Message, MessageEnvelope};
use crate::behaviour::complex::queue::{BehaviourScheduler, ScheduleStrategy};
use crate::behaviour::parallel::{FinishStrategy, ParallelBehaviourQueue};
use crate::behaviour::{BehaviourId, IntoBehaviour};
use crate::container::AgentLike;
use crate::context::{ContainerContext, Context};

pub(crate) use self::ams::AmsAgent;

pub type Aid = Cow<'static, str>;

mod ams;

enum AgentState {
    Initiated,
    Active,
    Suspended,
    // TODO: Implement these.
    // Waiting,
    // Transit,
}

pub struct Agent<E> {
    pub(crate) name: String,
    inbox: (Sender<MessageEnvelope>, Receiver<MessageEnvelope>),
    behaviours: ParallelBehaviourQueue<E>,
    state: AgentState,
}

impl<E: 'static> Agent<E> {
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            inbox: uchan::channel(),
            behaviours: ParallelBehaviourQueue::new(FinishStrategy::Never),
            state: AgentState::Initiated,
        }
    }
}

impl<E: 'static> Agent<E> {
    pub fn with_behaviour<K>(mut self, behaviour: impl IntoBehaviour<K, Event = E>) -> Self {
        self.add_behaviour(behaviour);
        self
    }

    pub fn add_behaviour<K>(&mut self, behaviour: impl IntoBehaviour<K, Event = E>) -> BehaviourId {
        let behaviour = behaviour.into_behaviour();
        let id = behaviour.id();
        self.behaviours.schedule(behaviour, ScheduleStrategy::End);
        id
    }
}

impl<E: 'static> AgentLike for Agent<E> {
    fn update(&mut self, ctx: &mut ContainerContext) -> bool {
        use AgentState::*;

        match self.state {
            Initiated => {
                // First register the agent with the ams.
                self.state = Active;
                return false;
            }
            Active => (),
            Suspended => return false,
        }

        let mut context = Context::new(Vec::<Message>::with_capacity(0));
        self.behaviours.action(&mut context);

        if let Some(container_ctx) = context.container.take() {
            ctx.merge(container_ctx);
        }

        context.agent.should_remove
    }

    fn get_name(&self) -> Cow<str> {
        Cow::from(&self.name)
    }
}
