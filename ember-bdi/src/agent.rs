use alloc::borrow::Cow;
use alloc::vec::Vec;

use ember_core::agent::Agent;
use ember_core::environment::Environment;
use ember_core::message::content::ember_bdil::BdilContent;
use ember_core::message::{Content, Message, MessageFilter, Performative};
use ember_fipa::agent::{ExecutionState, FipaAgent};

use crate::context::Context;
use crate::event::EventSource;
use crate::event::queue::EventQueue;
use crate::event::selector::FirstEvent;
use crate::intention::IntentionId;
use crate::intention::queue::{Fifo, IntentionQueue};
use crate::knowledge::base::KnowledgeBase;
use crate::literal::Literal;
use crate::plan::action::{Execute, PendingAction};
use crate::plan::library::PlanLibrary;
use crate::plan::selector::FirstApplicable;
use crate::plan::{GoalKind, Trigger, TriggeringEvent};
use crate::sensor::{Percept, Perceptor, Sensor};

#[derive(Debug)]
pub struct BdiAgent<'s, State, Action, Percept> {
    name: Cow<'static, str>,
    state: State,
    beliefs: KnowledgeBase,
    plans: PlanLibrary<Action>,
    intentions: IntentionQueue<Action>,
    /// Actions that returned pending on their last poll, keyed by the intention they belong to.
    /// Retried once per tick until they complete; their owning intention stays blocked in
    /// `intentions` for as long as they're here.
    pending_actions: Vec<(IntentionId, PendingAction<Action>)>,
    event_queue: EventQueue,
    sensors: Option<Vec<Sensor<'s, Percept>>>,
    fipa: FipaAgent,
}

impl<'a, State, Action, P> BdiAgent<'a, State, Action, P>
where
    P: Percept,
{
    pub fn with_sensor<S>(mut self, sensor: S) -> Self
    where
        S: Perceptor<Percept = P> + 'a,
    {
        self.add_sensor(sensor);
        self
    }

    pub fn add_sensor<S>(&mut self, sensor: S)
    where
        S: Perceptor<Percept = P> + 'a,
    {
        self.sensors
            .get_or_insert_default()
            .push(Sensor::new(sensor));
    }
}

impl<'s, State, Action, Percept> BdiAgent<'s, State, Action, Percept>
where
    Action: Clone,
{
    pub fn new(
        name: impl Into<Cow<'static, str>>,
        state: State,
        beliefs: Option<KnowledgeBase>,
        plans: PlanLibrary<Action>,
        initial_goals: impl IntoIterator<Item = Literal>,
    ) -> Self {
        let mut this = Self {
            name: name.into(),
            state,
            beliefs: beliefs.unwrap_or_default(),
            plans,
            intentions: IntentionQueue::default(),
            pending_actions: Vec::new(),
            event_queue: EventQueue::default(),
            sensors: None,
            fipa: FipaAgent::default(),
        };
        initial_goals.into_iter().for_each(|g| {
            this.handle_event(
                TriggeringEvent {
                    trigger: Trigger::Addition,
                    event: g,
                    goal: Some(GoalKind::Achieve),
                },
                EventSource::External,
            )
        });
        this
    }

    fn handle_event(&mut self, event: TriggeringEvent, source: EventSource) {
        if event.goal.is_none() {
            let ground = event.event.clone();

            match event.trigger {
                Trigger::Addition => self.beliefs.assert_no_event(ground),
                Trigger::Deletion => self.beliefs.remove_no_event(ground),
            };
        }

        let Some((plan, bindings)) = self.plans.select(&event, &self.beliefs, FirstApplicable)
        else {
            return;
        };

        let existing_intention = match source {
            EventSource::Internal(intention) => Some(intention),
            EventSource::External => None,
        };
        self.intentions
            .push(plan, bindings, existing_intention, event.clone());
    }

    fn handle_message(&mut self, performative: Performative, content: BdilContent) {
        match content {
            BdilContent::Literal(l) => {
                let literal = Literal::from(l);
                let (trigger, goal) = match performative {
                    Performative::Inform => (Trigger::Addition, None),
                    Performative::NotUnderstood => (Trigger::Addition, None),
                    _ => {
                        log::error!("unknown performative for bdil message");
                        return;
                    }
                };
                self.handle_event(
                    TriggeringEvent {
                        trigger,
                        event: literal,
                        goal,
                    },
                    EventSource::External,
                );
            }
        }
    }
}

impl<State, Action, P> BdiAgent<'_, State, Action, P>
where
    Action: Execute<State = State, Action = Action> + Clone,
    P: Percept,
{
    fn tick(&mut self, environment: &mut Environment) {
        let mut context = Context::new(environment);

        if let Some(sensors) = self.sensors.as_mut() {
            for sensor in sensors.iter_mut() {
                let Some(percept) = sensor.percept() else {
                    continue;
                };

                for (trigger, belief) in percept.into_beliefs().into_iter() {
                    let _ = match trigger {
                        Trigger::Addition => self.beliefs.assert(belief, &mut context),
                        Trigger::Deletion => self.beliefs.remove(belief, &mut context),
                    };
                }
            }
        }

        while let Some(message) =
            context.receive_message(Some(MessageFilter::language("ember-bdil").into()))
        {
            let Message {
                performative,
                content: Some(Content::Bdil(content)),
                ..
            } = message
            else {
                log::warn!("INTERNAL: bdi agent has incorrect mesage filter");
                continue;
            };

            self.handle_message(performative, content);
        }

        if let Some((event, source)) = self.event_queue.next_event(FirstEvent) {
            self.handle_event(event, source);
        }

        for (intention_id, pending) in core::mem::take(&mut self.pending_actions) {
            match pending.execute(&mut context, &mut self.state) {
                Some(pending) => self.pending_actions.push((intention_id, pending)),
                None => self.intentions.unblock(intention_id),
            }
        }

        let bindings = self.intentions.step(&mut Fifo, &mut context).into_owned();

        while let Some((intention_id, action)) = context.actions.pop() {
            use crate::plan::Action::*;
            let pending = match action {
                Builtin(action) => action.execute(&bindings, &mut context).map(Builtin),
                User(action) => action
                    .execute(&bindings, &mut context, &mut self.state)
                    .map(User),
            };

            if let Some(action) = pending {
                self.intentions.block(intention_id);
                self.pending_actions
                    .push((intention_id, PendingAction::new(action, bindings.clone())));
            }
        }

        context.events.into_iter().for_each(|(source, event)| {
            self.event_queue.push(event, source);
        });
    }
}

impl<State, Action, P> Agent for BdiAgent<'_, State, Action, P>
where
    Action: Execute<State = State, Action = Action> + Clone,
    P: Percept,
{
    fn update(&mut self, environment: &mut Environment) -> bool {
        match self.fipa.update(environment, &self.name) {
            ExecutionState::Initiated => return false,
            ExecutionState::Active => self.tick(environment),
        }
        self.intentions.is_empty()
    }

    fn get_name(&self) -> Cow<str> {
        self.name.clone()
    }
}

#[cfg(test)]
mod tests {
    use alloc::collections::VecDeque;
    use alloc::vec;

    use crate::bindings::BindingLookup;
    use crate::plan::{Action, BuiltinAction, Formula};
    use crate::testing::{literal, plan, trigger};

    use super::*;

    /// A test-only action with one variant that needs several polls to complete (`Wait`) and
    /// one that completes immediately (`Log`), so tests can observe both multi-poll behaviour
    /// and that it doesn't affect single-shot actions.
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    enum TestAction {
        Wait(u32),
        Log(&'static str),
    }

    impl Execute for TestAction {
        type State = Vec<&'static str>;
        type Action = TestAction;

        fn execute(
            self,
            _bindings: &impl BindingLookup,
            _context: &mut Context<Self::Action>,
            state: &mut Self::State,
        ) -> Option<Self> {
            match self {
                TestAction::Wait(remaining) => {
                    state.push("poll");
                    if remaining == 0 {
                        None
                    } else {
                        Some(TestAction::Wait(remaining - 1))
                    }
                }
                TestAction::Log(msg) => {
                    state.push(msg);
                    None
                }
            }
        }
    }

    fn new_environment() -> Environment {
        Environment::new(VecDeque::with_capacity(0))
    }

    #[test]
    fn test_multi_poll_action_blocks_its_own_intention_but_not_others() {
        let mut lib = PlanLibrary::default();
        lib.add(plan(
            trigger("wait_test", vec![], Some(GoalKind::Achieve)),
            None,
            vec![
                Formula::Action(Action::User(TestAction::Wait(2))),
                Formula::Action(Action::User(TestAction::Log("after"))),
            ],
        ));
        lib.add(plan(
            trigger("other_test", vec![], Some(GoalKind::Achieve)),
            None,
            vec![Formula::Action(Action::User(TestAction::Log("other")))],
        ));

        let mut agent = BdiAgent::<Vec<&'static str>, TestAction, ()>::new(
            "test-agent",
            Vec::new(),
            None,
            lib,
            vec![literal("wait_test", vec![]), literal("other_test", vec![])],
        );

        let mut environment = new_environment();

        // Tick 1: `Wait(2)` is dispatched and polled once. It doesn't complete, so its
        // intention is blocked and the action is kept around to be retried.
        agent.tick(&mut environment);
        assert_eq!(agent.state, vec!["poll"]);
        assert_eq!(agent.pending_actions.len(), 1);

        // Tick 2: the blocked intention is skipped by the scheduler, so `Wait` is only
        // retried (still pending) - it does *not* get to run its next formula (`Log("after")`).
        // Meanwhile the unrelated intention is free to run and completes its one action.
        agent.tick(&mut environment);
        assert_eq!(agent.state, vec!["poll", "poll", "other"]);
        assert_eq!(agent.pending_actions.len(), 1);

        // Tick 3: `Wait`'s last poll completes it, unblocking its intention, which then
        // immediately advances to `Log("after")` in the same tick.
        agent.tick(&mut environment);
        assert_eq!(agent.state, vec!["poll", "poll", "other", "poll", "after"]);
        assert!(agent.pending_actions.is_empty());

        // No actions are left to run; ticking further should not change the log, and the
        // agent should eventually report having no more intentions to work on.
        for _ in 0..10 {
            if agent.intentions.is_empty() {
                break;
            }
            agent.tick(&mut environment);
        }
        assert!(agent.intentions.is_empty());
        assert_eq!(agent.state, vec!["poll", "poll", "other", "poll", "after"]);
    }

    #[test]
    fn test_builtin_wait_blocks_its_intention_until_it_completes() {
        let mut lib = PlanLibrary::default();
        lib.add(plan(
            trigger("wait_test", vec![], Some(GoalKind::Achieve)),
            None,
            vec![
                Formula::Action(Action::Builtin(BuiltinAction::wait(
                    core::time::Duration::from_millis(0),
                ))),
                Formula::Action(Action::User(TestAction::Log("after"))),
            ],
        ));

        let mut agent = BdiAgent::<Vec<&'static str>, TestAction, ()>::new(
            "wait-agent",
            Vec::new(),
            None,
            lib,
            vec![literal("wait_test", vec![])],
        );

        let mut environment = new_environment();

        // Tick 1: `.wait` is dispatched. Its first poll only records the start time and is
        // always pending, so its intention must be blocked and the action kept for retry -
        // `Log("after")` must not run yet.
        agent.tick(&mut environment);
        assert!(agent.state.is_empty());
        assert_eq!(agent.pending_actions.len(), 1);

        // Tick 2: the interval (0ms) has elapsed, so `.wait`'s second poll completes it,
        // unblocking the intention, which then immediately advances to `Log("after")`.
        agent.tick(&mut environment);
        assert_eq!(agent.state, vec!["after"]);
        assert!(agent.pending_actions.is_empty());
    }
}
