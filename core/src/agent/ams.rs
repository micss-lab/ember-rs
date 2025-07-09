use alloc::borrow::Cow;
use alloc::collections::vec_deque::VecDeque;
use alloc::vec::Vec;

use crate::acl::message::{MessageFilter, Performative};
use crate::adt::{Adt, AgentReference};
use crate::behaviour::parallel::{FinishStrategy, ParallelBehaviourQueue};
use crate::behaviour::{CyclicBehaviour, OneShotBehaviour};
use crate::container::AgentLike;
use crate::context::{ContainerContext, Context};
use crate::fipa::{ActionKind, AmsAgentDescription, ManagementOntology};

use super::Aid;

pub(crate) struct AmsAgent {
    /// Inner agent on which ams behaviours will be stored.
    behaviours: ParallelBehaviourQueue<(), ActionKind>,
    /// Queue of actions to be performed using the hosting platform.
    actions: VecDeque<ActionKind>,
}

impl AgentLike for AmsAgent {
    fn update(&mut self, ctx: &mut ContainerContext) -> bool {
        use crate::behaviour::complex::scheduler::BehaviourScheduler;

        // log::trace!("Ticking ams agent");

        let mut context = Context::new_using_container(&mut *ctx);
        self.behaviours.action(&mut context, &mut ());

        // Do nothing with the container context for now.
        // if let Some(container_ctx) = context.container.take() {
        //     ctx.merge(container_ctx);
        // }

        while let Some(event) = context.local.events.pop() {
            self.actions.push_back(event);
        }

        // Store unhandled messages for the next request.
        ctx.merge(context.container);

        false
    }

    fn get_name(&self) -> Cow<str> {
        Cow::Borrowed("ams")
    }

    fn get_aid(&self) -> Aid {
        Aid::ams()
    }
}

impl AmsAgent {
    pub(crate) fn new() -> Self {
        let behaviours = ParallelBehaviourQueue::new_empty(FinishStrategy::Never)
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

        let aid: Aid = match agent.name.map(|n| n.parse()) {
            Some(Ok(aid)) => aid,
            Some(Err(e)) => {
                log::error!("Cannot register agent: {}", e);
                return;
            }
            None => {
                log::error!("Cannot register an agent without a name.");
                return;
            }
        };
        log::trace!("Trying to registering agent `{}`.", aid);
        match adt.entry(aid.clone()) {
            Entry::Vacant(entry) => {
                entry.insert(AgentReference { inbox: Vec::new() });
                log::info!("Agent `{}` successfully registered.", aid);
            }
            Entry::Occupied(_) => {
                log::error!(
                    "Cannot register agent `{}` as it is already registered.",
                    aid
                );
            }
        }
    }
}

struct StartupMessage;

impl OneShotBehaviour for StartupMessage {
    type AgentState = ();

    type Event = ActionKind;

    fn action(&self, _: &mut Context<Self::Event>, _: &mut Self::AgentState) {
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
    type AgentState = ();

    type Event = ActionKind;

    fn action(&mut self, ctx: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        let Some(message) = ctx.receive_message(Some(&self.filter)) else {
            log::trace!("Blocking agent management behaviour");
            ctx.block_behaviour();
            return;
        };
        let action_kind = match ManagementOntology::decode_message(message) {
            Ok(k) => k,
            Err(e) => {
                log::error!("Could not decode message: {:?}", e);
                todo!()
            }
        };
        ctx.emit_event(action_kind);
    }

    fn is_finished(&self) -> bool {
        false
    }
}
