use alloc::borrow::Cow;
use alloc::collections::vec_deque::VecDeque;
use alloc::string::ToString;
use alloc::vec::Vec;

use ember_core::agent::Agent as AgentTrait;
use ember_core::agent::aid::Aid;
use ember_core::behaviour::complex::parallel::{FinishStrategy, ParallelBehaviourQueue};
use ember_core::behaviour::simple::cyclic::CyclicBehaviour;
use ember_core::behaviour::simple::oneshot::OneShotBehaviour;
use ember_core::context::{ContainerContext, Context};
use ember_core::message::Performative;
use ember_core::message::filter::MessageFilter;

use crate::adt::{Adt, AgentReference, LocalAgentReference};
use crate::fipa::{ActionKind, AmsAgentDescription, ManagementOntology};

pub(crate) struct AmsAgent {
    /// Inner agent on which ams behaviours will be stored.
    behaviours: ParallelBehaviourQueue<(), ActionKind>,
    /// Queue of actions to be performed using the hosting platform.
    actions: VecDeque<ActionKind>,
}

impl AgentTrait for AmsAgent {
    fn update(&mut self, ctx: &mut ContainerContext) -> bool {
        use ember_core::behaviour::complex::scheduler::BehaviourScheduler;

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
                log::error!("Cannot register agent: {e}");
                return;
            }
            None => {
                log::error!("Cannot register an agent without a name.");
                return;
            }
        };
        if !aid.is_local() {
            log::error!("Cannot register agent that is not local to the ams.");
        }
        let name = aid.local_name().to_string();
        log::trace!("Trying to registering agent `{name}`.");
        match adt.entry(name.clone()) {
            Entry::Vacant(entry) => {
                entry.insert(AgentReference::Local(LocalAgentReference {
                    inbox: Vec::new(),
                }));
                log::info!("Agent `{}` successfully registered.", &name);
            }
            Entry::Occupied(_) => {
                log::error!(
                    "Cannot register agent `{aid}` as it is already registered."
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
                MessageFilter::ontology(ManagementOntology::name()),
            ]),
        }
    }
}

impl CyclicBehaviour for FipaAgentManagementBehaviour {
    type AgentState = ();

    type Event = ActionKind;

    fn action(&mut self, ctx: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        let Some(message) = ctx.receive_message(Some(Cow::Borrowed(&self.filter))) else {
            log::trace!("Blocking agent management behaviour");
            ctx.block_behaviour();
            return;
        };
        let action_kind = match ManagementOntology::decode_message(message) {
            Ok(k) => k,
            Err(e) => {
                log::error!("Could not decode message: {e:?}");
                todo!()
            }
        };
        ctx.emit_event(action_kind);
    }

    fn is_finished(&self) -> bool {
        false
    }
}
