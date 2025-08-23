use alloc::borrow::Cow;
use alloc::format;
use alloc::string::{String, ToString};

use ember_core::agent::Agent as AgentTrait;
use ember_core::agent::aid::Aid;
use ember_core::behaviour::complex::parallel::{FinishStrategy, ParallelBehaviourQueue};
use ember_core::behaviour::{BehaviourId, IntoBehaviour};
use ember_core::context::{ContainerContext, Context};
use ember_core::message::{Message, MessageEnvelope, Performative};

use crate::fipa::{self, AmsAgentDescription, RegisterAction};

pub(crate) use self::ams::AmsAgent;

mod ams;

enum ExecutionState {
    Initiated,
    Active,
    // TODO: Implement these.
    // Suspended,
    // Waiting,
    // Transit,
}

pub struct Agent<'a, S, E> {
    pub(crate) name: String,
    behaviours: ParallelBehaviourQueue<'a, S, E>,
    execution_state: ExecutionState,
    state: S,
}

impl<S, E> Agent<'_, S, E> {
    pub fn new(name: impl ToString, state: S) -> Self {
        Self {
            name: name.to_string(),
            behaviours: ParallelBehaviourQueue::new_empty(FinishStrategy::Never),
            execution_state: ExecutionState::Initiated,
            state,
        }
    }
}

impl<'a, S, E> Agent<'a, S, E> {
    pub fn with_behaviour<K>(
        mut self,
        behaviour: impl IntoBehaviour<'a, K, AgentState = S, Event = E>,
    ) -> Self {
        self.add_behaviour(behaviour);
        self
    }

    pub fn add_behaviour<K>(
        &mut self,
        behaviour: impl IntoBehaviour<'a, K, AgentState = S, Event = E>,
    ) -> BehaviourId {
        let behaviour = behaviour.into_behaviour();
        let id = behaviour.id();
        self.behaviours.add_behaviour(behaviour);
        id
    }
}

impl<S, E> AgentTrait for Agent<'_, S, E> {
    fn update(&mut self, ctx: &mut ContainerContext) -> bool {
        use ExecutionState::*;
        use ember_core::behaviour::complex::scheduler::BehaviourScheduler;
        use ember_core::message::content::codec::AgentActionCodec;

        // log::trace!("Ticking agent `{}`", self.name);

        match self.execution_state {
            Initiated => {
                // First register the agent with the ams.
                let ams_aid = Aid::local("ams");

                ctx.send_message(MessageEnvelope::new(
                    ams_aid.clone(),
                    Message {
                        performative: Performative::Request,
                        sender: None,
                        receiver: ams_aid.into(),
                        reply_to: None,
                        ontology: Some(fipa::ManagementOntology::name().to_string()),
                        content: RegisterAction {
                            ams: AmsAgentDescription { name: None },
                            agent: AmsAgentDescription {
                                name: Some(format!("{}@local", self.get_name())),
                            },
                        }
                        .into_content()
                        .into(),
                    },
                ));
                log::debug!("Sending ams register request for agent `{}`.", self.name);
                self.execution_state = Active;
                return false;
            }
            Active => (),
        }

        let mut context = Context::new_using_container(&mut *ctx);
        self.behaviours.action(&mut context, &mut self.state);

        ctx.merge(context.container);

        context.agent.should_remove
    }

    fn get_name(&self) -> Cow<str> {
        Cow::from(&self.name)
    }
}
