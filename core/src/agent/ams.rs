use alloc::borrow::Cow;
use alloc::collections::vec_deque::VecDeque;
use alloc::vec::Vec;

use crate::acl::message::{Message, MessageFilter, Performative};
use crate::adt::Adt;
use crate::behaviour::complex::queue::BehaviourScheduler;
use crate::behaviour::parallel::{FinishStrategy, ParallelBehaviourQueue};
use crate::behaviour::{CyclicBehaviour, OneShotBehaviour};
use crate::container::AgentLike;
use crate::context::{ContainerContext, Context};
use crate::fipa::{ActionKind, AmsAgentDescription, ManagementOntology};

use super::Aid;

pub(crate) struct AmsAgent {
    /// Inner agent on which ams behaviours will be stored.
    behaviours: ParallelBehaviourQueue<ActionKind>,
    /// Queue of actions to be performed using the hosting platform.
    actions: VecDeque<ActionKind>,
}

impl AgentLike for AmsAgent {
    fn update(&mut self, _context: &mut ContainerContext) -> bool {
        let mut context = Context::new(Vec::<Message>::with_capacity(0));
        self.behaviours.action(&mut context);

        // Do nothing with the container context for now.
        // if let Some(container_ctx) = context.container.take() {
        //     ctx.merge(container_ctx);
        // }

        while let Some(event) = context.local.events.pop() {
            self.actions.push_back(event);
        }

        false
    }

    fn get_name(&self) -> Cow<str> {
        Cow::Borrowed("ams")
    }
}

impl AmsAgent {
    pub(crate) fn new() -> Self {
        let behaviours = ParallelBehaviourQueue::new(FinishStrategy::Never)
            .with_behaviour(StartupMessage)
            .with_behaviour(FipaAgentManagementBehaviour::new());
        Self {
            behaviours,
            actions: VecDeque::new(),
        }
    }

    pub(crate) fn perform_platform_actions(&mut self, adt: &mut Adt) {
        use ActionKind::*;

        while let Some(action) = self.actions.pop_front() {
            match action {
                Register(r) => self.register_agent(r.ams, r.agent, adt),
            }
        }
    }

    fn register_agent(
        &mut self,
        _ams: AmsAgentDescription,
        agent: AmsAgentDescription,
        adt: &mut Adt,
    ) {
        // TODO: Check that the ams for which the action is meant is this one.

        use alloc::collections::btree_map::Entry;

        let aid: Aid = match agent.name {
            Some(name) => Cow::Owned(name),
            None => {
                log::error!("Cannot register an agent without the name.");
                return;
            }
        };
        match adt.entry(aid) {
            Entry::Vacant(_) => {
                log::error!("Cannot register an agent that is not local to this platform.");
            }
            Entry::Occupied(mut entry) => {
                let agent = entry.get_mut();
                if agent.registered {
                    log::error!("Cannot register agent that has already been registered.");
                }
                agent.registered = true;
            }
        }
    }
}

struct StartupMessage;

impl OneShotBehaviour for StartupMessage {
    type Event = ActionKind;

    fn action(&self, _: &mut Context<Self::Event>) {
        log::debug!("Ams agent has registered.");
    }
}

struct FipaAgentManagementBehaviour {
    filter: MessageFilter,
}

impl FipaAgentManagementBehaviour {
    fn new() -> Self {
        Self {
            filter: MessageFilter::and([
                MessageFilter::performative(Performative::Request),
                MessageFilter::ontology(ManagementOntology::name().into()),
            ]),
        }
    }
}

impl CyclicBehaviour for FipaAgentManagementBehaviour {
    type Event = ActionKind;

    fn action(&mut self, ctx: &mut Context<Self::Event>) {
        let Some(message) = ctx.receive_message(Some(&self.filter)) else {
            ctx.block_behaviour();
            return;
        };
        let action_kind = match ManagementOntology::decode_message(message) {
            Ok(k) => k,
            Err(e) => todo!(),
        };
        ctx.emit_event(action_kind);
    }

    fn is_finished(&self) -> bool {
        false
    }
}
