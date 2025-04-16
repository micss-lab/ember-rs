use alloc::borrow::Cow;
use alloc::format;
use alloc::string::{String, ToString};

use crate::acl::message::{Message, MessageEnvelope, Performative};
use crate::behaviour::complex::queue::{BehaviourScheduler, ScheduleStrategy};
use crate::behaviour::parallel::{FinishStrategy, ParallelBehaviourQueue};
use crate::behaviour::{BehaviourId, IntoBehaviour};
use crate::container::AgentLike;
use crate::context::{ContainerContext, Context};
use crate::fipa::{self, AmsAgentDescription, RegisterAction};

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
    behaviours: ParallelBehaviourQueue<E>,
    state: AgentState,
}

impl<E: 'static> Agent<E> {
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
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
        use crate::acl::codec::AgentActionCodec;
        use AgentState::*;

        log::trace!("Ticking agent `{}`", self.name);

        match self.state {
            Initiated => {
                // First register the agent with the ams.
                let ams_aid = Cow::Borrowed("ams@local");

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
                                name: Some(format!("{}@local", self.get_name().to_string())),
                            },
                        }
                        .into_content()
                        .into(),
                    },
                ));
                log::debug!("Sending ams register request.");
                self.state = Active;
                return false;
            }
            Active => (),
            Suspended => return false,
        }

        let mut context = Context::new_using_container(&mut *ctx);
        self.behaviours.action(&mut context);

        ctx.merge(context.container);

        context.agent.should_remove
    }

    fn get_name(&self) -> Cow<str> {
        Cow::from(&self.name)
    }
}
