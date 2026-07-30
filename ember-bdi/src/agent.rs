use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::collections::VecDeque;

use ember_core::agent::Agent;
use ember_core::environment::Environment;
use ember_core::message::content::ember_bdil::BdilContent;
use ember_core::message::{Content, Message, MessageFilter, Performative};
use ember_fipa::agent::{ExecutionState, FipaAgent};

use crate::context::Context;
use crate::event::EventSource;
use crate::event::queue::EventQueue;
use crate::event::selector::{EventSelector, FirstEvent};
use crate::intention::IntentionId;
use crate::intention::queue::{Fifo, IntentionQueue, Scheduler};
use crate::knowledge::base::KnowledgeBase;
use crate::literal::Literal;
use crate::plan::action::{Execute, PendingAction};
use crate::plan::library::PlanLibrary;
use crate::plan::selector::{FirstApplicable, PlanSelector};
use crate::plan::{GoalKind, Trigger, TriggeringEvent};
use crate::sensor::{Percept, Perceptor, Sensor};
use crate::term::{Structure, Term};

/// Limits on how much work a single [`BdiAgent::update`] tick may perform, so a busy agent
/// degrades gracefully (falls behind smoothly) instead of doing an unbounded amount of work
/// (and therefore taking unbounded time) in a single tick.
///
/// Every field defaults to today's unconfigured behaviour, so adopting a `TickBudget` is
/// opt-in and only ever *adds* limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickBudget {
    /// Maximum number of inbound `ember-bdil` messages handled per tick. `None` means every
    /// message currently queued is handled, however many there are.
    pub max_messages: Option<usize>,
    /// Maximum number of events (belief/goal additions or deletions) handled per tick. Must be
    /// at least `1` for the agent to make progress; defaults to `1`, matching the un-configured
    /// behaviour.
    pub max_events: usize,
    /// Maximum number of sensors polled per tick, round-robin across ticks so no sensor is
    /// starved. `None` means every sensor is polled every tick.
    pub max_sensors: Option<usize>,
    /// Maximum number of blocked/multi-tick actions retried per tick, round-robin across ticks.
    /// `None` means every pending action is retried every tick.
    pub max_pending_actions: Option<usize>,
    /// Maximum number of intentions stepped per tick. Each stepped intention's own actions are
    /// always fully executed (using that intention's own bindings) before the next intention is
    /// stepped, so raising this is safe: it can never mix up bindings between intentions.
    /// Defaults to `1`, matching the un-configured behaviour.
    pub max_intentions: usize,
}

impl Default for TickBudget {
    fn default() -> Self {
        Self {
            max_messages: None,
            max_events: 1,
            max_sensors: None,
            max_pending_actions: None,
            max_intentions: 1,
        }
    }
}

#[derive(Debug)]
pub struct BdiAgent<
    's,
    State,
    Action,
    Percept,
    Sched = Fifo,
    Sel = FirstEvent,
    PSel = FirstApplicable,
> {
    name: Cow<'static, str>,
    state: State,
    beliefs: KnowledgeBase,
    plans: PlanLibrary<Action, PSel>,
    intentions: IntentionQueue<Action, Sched>,
    /// Actions that returned pending on their last poll, keyed by the intention they belong to.
    /// Retried until they complete (subject to `tick_budget.max_pending_actions`); their owning
    /// intention stays blocked in `intentions` for as long as they're here. A `VecDeque` so
    /// retries can round-robin: serviced-and-still-pending entries move to the back, untouched
    /// ones stay at the front and are tried first next tick.
    pending_actions: VecDeque<(IntentionId, PendingAction<Action>)>,
    event_queue: EventQueue<Sel>,
    /// A `VecDeque` for the same round-robin reason as `pending_actions`: polled sensors rotate
    /// to the back so `tick_budget.max_sensors` doesn't starve the ones later in the list.
    sensors: Option<VecDeque<Sensor<'s, Percept>>>,
    fipa: FipaAgent,
    tick_budget: TickBudget,
}

impl<'a, State, Action, P, Sched, Sel, PSel> BdiAgent<'a, State, Action, P, Sched, Sel, PSel>
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
            .push_back(Sensor::new(sensor));
    }
}

impl<'s, State, Action, Percept, Sched, Sel, PSel>
    BdiAgent<'s, State, Action, Percept, Sched, Sel, PSel>
{
    /// Configures how much work a single tick may perform. See [`TickBudget`].
    pub fn with_tick_budget(mut self, budget: TickBudget) -> Self {
        self.tick_budget = budget;
        self
    }

    /// Configures which intention is stepped next when several are runnable. Replaces the
    /// default (`Fifo`). A different scheduler is a different type, so this returns a
    /// differently-typed agent rather than mutating in place.
    pub fn with_scheduler<NewSched>(
        self,
        scheduler: NewSched,
    ) -> BdiAgent<'s, State, Action, Percept, NewSched, Sel, PSel> {
        BdiAgent {
            name: self.name,
            state: self.state,
            beliefs: self.beliefs,
            plans: self.plans,
            intentions: self.intentions.with_scheduler(scheduler),
            pending_actions: self.pending_actions,
            event_queue: self.event_queue,
            sensors: self.sensors,
            fipa: self.fipa,
            tick_budget: self.tick_budget,
        }
    }

    /// Configures which queued event is handled next. Replaces the default (`FirstEvent`). A
    /// different selector is a different type, so this returns a differently-typed agent rather
    /// than mutating in place.
    pub fn with_event_selector<NewSel>(
        self,
        selector: NewSel,
    ) -> BdiAgent<'s, State, Action, Percept, Sched, NewSel, PSel> {
        BdiAgent {
            name: self.name,
            state: self.state,
            beliefs: self.beliefs,
            plans: self.plans,
            intentions: self.intentions,
            pending_actions: self.pending_actions,
            event_queue: self.event_queue.with_event_selector(selector),
            sensors: self.sensors,
            fipa: self.fipa,
            tick_budget: self.tick_budget,
        }
    }

    /// Configures how a plan is chosen among those applicable to an event. Replaces the default
    /// (`FirstApplicable`). A different selector is a different type, so this returns a
    /// differently-typed agent rather than mutating in place.
    pub fn with_plan_selector<NewPSel>(
        self,
        selector: NewPSel,
    ) -> BdiAgent<'s, State, Action, Percept, Sched, Sel, NewPSel> {
        BdiAgent {
            name: self.name,
            state: self.state,
            beliefs: self.beliefs,
            plans: self.plans.with_plan_selector(selector),
            intentions: self.intentions,
            pending_actions: self.pending_actions,
            event_queue: self.event_queue,
            sensors: self.sensors,
            fipa: self.fipa,
            tick_budget: self.tick_budget,
        }
    }
}

impl<'s, State, Action, Percept, Sched, Sel, PSel>
    BdiAgent<'s, State, Action, Percept, Sched, Sel, PSel>
where
    Action: Clone,
    Sched: Default,
    Sel: Default,
    PSel: PlanSelector<Action>,
{
    pub fn new(
        name: impl Into<Cow<'static, str>>,
        state: State,
        beliefs: Option<KnowledgeBase>,
        plans: PlanLibrary<Action, PSel>,
        initial_goals: impl IntoIterator<Item = Literal>,
    ) -> Self {
        let mut this = Self {
            name: name.into(),
            state,
            beliefs: beliefs.unwrap_or_default(),
            plans,
            intentions: IntentionQueue::default(),
            pending_actions: VecDeque::new(),
            event_queue: EventQueue::default(),
            sensors: None,
            fipa: FipaAgent::default(),
            tick_budget: TickBudget::default(),
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
}

impl<'s, State, Action, Percept, Sched, Sel, PSel>
    BdiAgent<'s, State, Action, Percept, Sched, Sel, PSel>
where
    Action: Clone,
    PSel: PlanSelector<Action>,
{
    fn handle_event(&mut self, event: TriggeringEvent, source: EventSource) {
        if event.goal.is_none() {
            let ground = event.event.clone();

            match event.trigger {
                Trigger::Addition => self.beliefs.assert_no_event(ground),
                Trigger::Deletion => self.beliefs.remove_no_event(ground),
            };
        }

        let Some((plan, bindings)) = self.plans.select(&event, &self.beliefs) else {
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
                let literal = {
                    let literal = Literal::from(l);
                    Literal {
                        negated: false,
                        structure: Structure {
                            functor: "message".into(),
                            arguments: Some(Box::new([
                                Term::String(performative.as_str().into()),
                                Term::Literal(literal),
                            ])),
                        },
                    }
                };

                self.handle_event(
                    TriggeringEvent {
                        trigger: Trigger::Addition,
                        event: literal,
                        goal: None,
                    },
                    EventSource::External,
                );
            }
        }
    }
}

impl<State, Action, P, Sched, Sel, PSel> BdiAgent<'_, State, Action, P, Sched, Sel, PSel>
where
    Action: Execute<State = State, Action = Action> + Clone,
    P: Percept,
    Sched: Scheduler<Action>,
    Sel: EventSelector,
    PSel: PlanSelector<Action>,
{
    fn tick(&mut self, environment: &mut Environment) {
        let mut context = Context::new(environment);

        // Sensors: rotate through a `VecDeque` so a `max_sensors` cap can't starve the sensors
        // later in the list - a sensor is always moved to the back after being polled,
        // regardless of `max_sensors`, so untouched ones stay at the front for next tick.
        if let Some(sensors) = self.sensors.as_mut() {
            let take = self
                .tick_budget
                .max_sensors
                .unwrap_or(sensors.len())
                .min(sensors.len());

            for _ in 0..take {
                let Some(mut sensor) = sensors.pop_front() else {
                    break;
                };

                if let Some(percept) = sensor.percept() {
                    for (trigger, belief) in percept.into_beliefs().into_iter() {
                        let _ = match trigger {
                            Trigger::Addition => self.beliefs.assert(belief, &mut context),
                            Trigger::Deletion => self.beliefs.remove(belief, &mut context),
                        };
                    }
                }

                sensors.push_back(sensor);
            }
        }

        // Messages: bounded so a burst of inbound messages can't stall the rest of the tick.
        // `receive_message` just pops from the inbox, so anything left over waits for next tick.
        let max_messages = self.tick_budget.max_messages.unwrap_or(usize::MAX);
        for _ in 0..max_messages {
            let Some(message) =
                context.receive_message(Some(MessageFilter::language("ember-bdil").into()))
            else {
                break;
            };

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

        // Events: bounded so a backlog can be drained faster than one-per-tick when the budget
        // allows it, instead of always falling further behind under load.
        for _ in 0..self.tick_budget.max_events {
            let Some((event, source)) = self.event_queue.next_event() else {
                break;
            };
            self.handle_event(event, source);
        }

        // Pending (blocked/multi-tick) actions: same round-robin rotation as sensors.
        let take = self
            .tick_budget
            .max_pending_actions
            .unwrap_or(self.pending_actions.len())
            .min(self.pending_actions.len());

        for _ in 0..take {
            let Some((intention_id, pending)) = self.pending_actions.pop_front() else {
                break;
            };

            match pending.execute(&mut context, &self.beliefs, &mut self.state) {
                Some(pending) => self.pending_actions.push_back((intention_id, pending)),
                None => self.intentions.unblock(intention_id),
            }
        }

        // Intentions: step up to `max_intentions` of them. Each stepped intention's own actions
        // are fully drained and executed - with *its own* bindings - before the next intention
        // is stepped, so `context.actions` is always empty going into a step. That's what makes
        // raising `max_intentions` above 1 safe: an action can never be executed against a
        // different intention's bindings, because no other intention's actions can be sitting in
        // the queue at the same time.
        for _ in 0..self.tick_budget.max_intentions {
            if !self.intentions.has_runnable() {
                break;
            }

            let bindings = self.intentions.step(&mut context).into_owned();

            while let Some((intention_id, action)) = context.actions.pop() {
                use crate::plan::Action::*;
                let pending = match action {
                    Builtin(action) => action
                        .execute(&bindings, &mut context, &self.beliefs)
                        .map(Builtin),
                    User(action) => action
                        .execute(&bindings, &mut context, &self.beliefs, &mut self.state)
                        .map(User),
                };

                if let Some(action) = pending {
                    self.intentions.block(intention_id);
                    self.pending_actions
                        .push_back((intention_id, PendingAction::new(action, bindings.clone())));
                }
            }
        }

        context.events.into_iter().for_each(|(source, event)| {
            self.event_queue.push(event, source);
        });
    }
}

impl<State, Action, P, Sched, Sel, PSel> Agent for BdiAgent<'_, State, Action, P, Sched, Sel, PSel>
where
    Action: Execute<State = State, Action = Action> + Clone,
    P: Percept,
    Sched: Scheduler<Action>,
    Sel: EventSelector,
    PSel: PlanSelector<Action>,
{
    fn update(&mut self, environment: &mut Environment) -> bool {
        match self.fipa.update(environment, &self.name) {
            ExecutionState::Initiated => return false,
            ExecutionState::Active => self.tick(environment),
        }
        false
    }

    fn get_name(&self) -> Cow<str> {
        self.name.clone()
    }
}

#[cfg(test)]
mod tests {
    use alloc::collections::VecDeque;
    use alloc::string::ToString;
    use alloc::vec;
    use alloc::vec::Vec;

    use crate::bindings::BindingLookup;
    use crate::knowledge::query::IntoQuery;

    use crate::plan::{Action, BuiltinAction, Formula};
    use crate::testing::{assert_belief, literal, literal_formula, plan, string, trigger, variable, variable_term};
    use crate::variable::Variable;

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
            _knowledge: &KnowledgeBase,
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

    #[test]
    fn test_forall_spawns_independent_intentions_without_blocking_the_calling_plan() {
        let mut lib = PlanLibrary::<TestAction>::default();

        let x = variable();
        lib.add(plan(
            trigger("start", vec![], Some(GoalKind::Achieve)),
            None,
            vec![
                Formula::Action(Action::Builtin(BuiltinAction::Forall {
                    query: literal_formula("item", vec![variable_term(&x)]),
                    goal: literal("mark_processed", vec![variable_term(&x)]),
                })),
                Formula::Action(Action::User(TestAction::Log("after_forall"))),
            ],
        ));

        let y = variable();
        lib.add(plan(
            trigger(
                "mark_processed",
                vec![variable_term(&y)],
                Some(GoalKind::Achieve),
            ),
            None,
            vec![Formula::Belief {
                trigger: Trigger::Addition,
                belief: literal("processed", vec![variable_term(&y)]),
            }],
        ));

        let mut beliefs = KnowledgeBase::default();
        assert_belief(&mut beliefs, "item", vec![string("a")]);
        assert_belief(&mut beliefs, "item", vec![string("b")]);
        assert_belief(&mut beliefs, "item", vec![string("c")]);

        let mut agent = BdiAgent::<Vec<&'static str>, TestAction, ()>::new(
            "forall-agent",
            Vec::new(),
            Some(beliefs),
            lib,
            vec![literal("start", vec![])],
        );

        let mut environment = new_environment();

        for _ in 0..20 {
            if agent.intentions.is_empty() {
                break;
            }
            agent.tick(&mut environment);
        }

        assert!(
            agent.intentions.is_empty(),
            "agent should reach quiescence: the main plan and all three spawned branches finish"
        );
        // The step after `.forall` in the calling plan must run exactly once - not once per
        // spawned branch - and it must not have waited for the branches to complete first.
        assert_eq!(agent.state, vec!["after_forall"]);

        for item in ["a", "b", "c"] {
            let query_formula = literal_formula("processed", vec![string(item)]);
            let mut query = (&query_formula).into_query(&agent.beliefs);
            assert!(
                query.next_bindings(None).is_some(),
                "processed({item}) should have been asserted by its own spawned intention"
            );
        }
    }

    /// A test-only action whose whole purpose is to resolve a variable through the bindings
    /// it's executed with and record what it actually saw. Used to catch the exact bug
    /// `max_intentions > 1` would reintroduce if actions were ever dispatched against the
    /// wrong intention's bindings.
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    struct RecordArg(Variable);

    impl Execute for RecordArg {
        type State = Vec<alloc::string::String>;
        type Action = RecordArg;

        fn execute(
            self,
            bindings: &impl BindingLookup,
            _context: &mut Context<Self::Action>,
            _knowledge: &KnowledgeBase,
            state: &mut Self::State,
        ) -> Option<Self> {
            let seen = bindings
                .lookup_as_type::<alloc::string::String>(&self.0)
                .and_then(Result::ok)
                .unwrap_or_else(|| "<unbound>".into());
            state.push(seen);
            None
        }
    }

    #[test]
    fn test_max_intentions_gives_each_stepped_intention_its_own_correct_bindings() {
        // Both intentions run the *same* plan (so they share the exact same `Variable`/
        // `VariableId` for `X` - the dangerous case where cross-contaminated bindings would
        // resolve to a plausible-looking but wrong value instead of an obviously-missing one).
        let x = variable();
        let mut lib = PlanLibrary::<RecordArg>::default();
        lib.add(plan(
            trigger("say", vec![variable_term(&x)], Some(GoalKind::Achieve)),
            None,
            vec![Formula::Action(Action::User(RecordArg(x.clone())))],
        ));

        // Both initial goals are turned into intentions immediately during `new` (they don't go
        // through the event queue at all), so both already exist before the first tick.
        let agent = BdiAgent::<Vec<alloc::string::String>, RecordArg, ()>::new(
            "budget-agent",
            Vec::new(),
            None,
            lib,
            vec![
                literal("say", vec![string("first")]),
                literal("say", vec![string("second")]),
            ],
        );
        // A single-action plan body actually takes two scheduler steps to fully retire (one to
        // run the action, one more to notice the body is now empty), and `Fifo` keeps
        // re-selecting the same intention until it's actually removed - so budget generously
        // rather than trying to hit an exact step count.
        let mut agent = agent.with_tick_budget(TickBudget {
            max_intentions: 10,
            ..TickBudget::default()
        });

        let mut environment = new_environment();
        agent.tick(&mut environment);

        let mut recorded = agent.state.clone();
        recorded.sort();
        assert_eq!(
            recorded,
            vec!["first".to_string(), "second".to_string()],
            "each intention's action must resolve its own binding, never the other's"
        );
        assert!(agent.intentions.is_empty());
    }

    /// A sensor that fires exactly once, producing a percept that expands into several belief
    /// additions in one shot - the only way to get more than one event queued up at the same
    /// time without needing several ticks (initial goals and inbound messages both bypass the
    /// event queue entirely, going straight through `handle_event`).
    struct BurstSensor(bool);

    impl Perceptor for BurstSensor {
        type Percept = Burst;

        fn percept(&mut self) -> Option<Burst> {
            core::mem::take(&mut self.0).then_some(Burst)
        }
    }

    struct Burst;

    impl Percept for Burst {
        fn into_beliefs(self) -> impl IntoIterator<Item = (Trigger, Literal)> {
            [
                (Trigger::Addition, literal("ping", vec![crate::testing::number(1.0)])),
                (Trigger::Addition, literal("ping", vec![crate::testing::number(2.0)])),
                (Trigger::Addition, literal("ping", vec![crate::testing::number(3.0)])),
            ]
        }
    }

    #[test]
    fn test_max_events_drains_a_burst_of_belief_events_in_one_tick() {
        let n = variable();
        let mut lib = PlanLibrary::<TestAction>::default();
        lib.add(plan(
            trigger("ping", vec![variable_term(&n)], None),
            None,
            vec![Formula::Action(Action::User(TestAction::Log("pong")))],
        ));

        let mut agent =
            BdiAgent::<Vec<&'static str>, TestAction, Burst>::new("event-budget-agent", Vec::new(), None, lib, vec![]);
        agent.add_sensor(BurstSensor(true));
        let mut agent = agent.with_tick_budget(TickBudget {
            max_events: 10,
            max_intentions: 10,
            ..TickBudget::default()
        });

        let mut environment = new_environment();

        // Tick 1: the sensor fires, asserting three new `ping/1` beliefs. Each queues an event,
        // but events queued during a tick are only flushed to the event queue at its end, so
        // none of them are handled yet.
        agent.tick(&mut environment);
        assert!(agent.state.is_empty());

        // Tick 2: with a raised event budget, all three queued events are turned into
        // intentions - and with a raised intention budget, all three run to completion - in
        // this single tick. With the default budget (1 of each) this would instead take
        // several ticks to fully drain.
        agent.tick(&mut environment);

        let mut state = agent.state.clone();
        state.sort();
        assert_eq!(state, vec!["pong", "pong", "pong"]);
        assert!(agent.intentions.is_empty());
    }

    #[test]
    fn test_custom_plan_selector_replaces_first_applicable() {
        // Reject any applicable plan with a one-step body; among these two same-triggered
        // plans, that rejects the first (added) one and forces the second to be selected -
        // `FirstApplicable` would have picked the first instead.
        struct RejectShortBody;
        impl<A> crate::plan::selector::PlanSelector<A> for RejectShortBody {
            fn filter_plan<'p>(&mut self, plan: &'p crate::plan::Plan<A>) -> Option<&'p crate::plan::Plan<A>> {
                (plan.body.len() > 1).then_some(plan)
            }
        }

        let mut lib = PlanLibrary::<TestAction>::default();
        lib.add(plan(
            trigger("start", vec![], Some(GoalKind::Achieve)),
            None,
            vec![Formula::Action(Action::User(TestAction::Log("first")))],
        ));
        lib.add(plan(
            trigger("start", vec![], Some(GoalKind::Achieve)),
            None,
            vec![
                Formula::Action(Action::User(TestAction::Log("second"))),
                Formula::Action(Action::User(TestAction::Log("marker"))),
            ],
        ));
        let lib = lib.with_plan_selector(RejectShortBody);

        let mut agent = BdiAgent::<
            Vec<&'static str>,
            TestAction,
            (),
            Fifo,
            FirstEvent,
            RejectShortBody,
        >::new(
            "selector-agent",
            Vec::new(),
            None,
            lib,
            vec![literal("start", vec![])],
        );

        let mut environment = new_environment();
        for _ in 0..10 {
            if agent.intentions.is_empty() {
                break;
            }
            agent.tick(&mut environment);
        }

        assert_eq!(agent.state, vec!["second", "marker"]);
    }
}
