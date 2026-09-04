use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::rc::Rc;

use ember_fipa::agent::{ExecutionState, FipaAgent};

use ember_core::agent::Agent;
use ember_core::environment::Environment;
use ember_core::message::content::ember_bdil::BdilContent;
use ember_core::message::{Content, Message, MessageFilter, Performative};

use crate::context::{Context, PureContext};
use crate::event::EventSource;
use crate::event::queue::EventQueue;
use crate::event::selector::{EventSelector, FirstEvent};
use crate::intention::IntentionId;
use crate::intention::queue::{IntentionQueue, Random, Scheduler};
use crate::knowledge::base::KnowledgeBase;
use crate::literal::Literal;
use crate::plan::action::{Execute, ExecuteResult, PendingAction};
use crate::plan::library::PlanLibrary;
use crate::plan::selector::{FirstApplicable, PlanSelector};
use crate::plan::{GoalKind, Trigger, TriggeringEvent};
use crate::sensor::{Percept, Perceptor, Sensor};
use crate::term::{Structure, Term};

#[derive(Debug)]
pub struct BdiAgent<
    's,
    State,
    Action,
    Percept,
    Sched = Random,
    Sel = FirstEvent,
    PSel = FirstApplicable,
> {
    name: PureContext,
    state: State,
    beliefs: KnowledgeBase,
    plans: Rc<PlanLibrary<Action>>,
    plan_selector: PSel,
    intentions: IntentionQueue<Action, Sched>,
    pending_actions: VecDeque<(Option<IntentionId>, PendingAction<Action>)>,
    event_queue: EventQueue<Sel>,
    sensors: Option<VecDeque<Sensor<'s, Percept>>>,
    fipa: FipaAgent,
    tick_budget: TickBudget,
}

impl<'s, State, Action, Percept, Sched, Sel, PSel>
    BdiAgent<'s, State, Action, Percept, Sched, Sel, PSel>
where
    Action: Clone,
    Sched: Default,
    Sel: Default,
    PSel: PlanSelector<Action> + Default,
{
    pub fn new(
        name: impl Into<Cow<'static, str>>,
        state: State,
        beliefs: Option<KnowledgeBase>,
        plans: impl Into<Rc<PlanLibrary<Action>>>,
        initial_goals: impl IntoIterator<Item = Literal>,
    ) -> Self {
        let mut this = Self {
            name: PureContext::new(Rc::new(name.into())),
            state,
            beliefs: beliefs.unwrap_or_default(),
            plans: plans.into(),
            plan_selector: PSel::default(),
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

    pub fn with_intention_scheduler<NewSched>(
        self,
        scheduler: NewSched,
    ) -> BdiAgent<'s, State, Action, Percept, NewSched, Sel, PSel> {
        BdiAgent {
            name: self.name,
            state: self.state,
            beliefs: self.beliefs,
            plans: self.plans,
            plan_selector: self.plan_selector,
            intentions: self.intentions.with_scheduler(scheduler),
            pending_actions: self.pending_actions,
            event_queue: self.event_queue,
            sensors: self.sensors,
            fipa: self.fipa,
            tick_budget: self.tick_budget,
        }
    }

    pub fn with_event_selector<NewSel>(
        self,
        selector: NewSel,
    ) -> BdiAgent<'s, State, Action, Percept, Sched, NewSel, PSel> {
        BdiAgent {
            name: self.name,
            state: self.state,
            beliefs: self.beliefs,
            plans: self.plans,
            plan_selector: self.plan_selector,
            intentions: self.intentions,
            pending_actions: self.pending_actions,
            event_queue: self.event_queue.with_event_selector(selector),
            sensors: self.sensors,
            fipa: self.fipa,
            tick_budget: self.tick_budget,
        }
    }

    pub fn with_plan_selector<NewPSel>(
        self,
        selector: NewPSel,
    ) -> BdiAgent<'s, State, Action, Percept, Sched, Sel, NewPSel> {
        BdiAgent {
            name: self.name,
            state: self.state,
            beliefs: self.beliefs,
            plans: self.plans,
            plan_selector: selector,
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
    PSel: PlanSelector<Action>,
{
    fn handle_event(&mut self, event: TriggeringEvent, source: EventSource) {
        // Free the intention that was blocked by this event not being handled.
        if let EventSource::Internal(i) = source {
            self.intentions.unblock_event(i);
        }

        if event.goal.is_none() {
            let ground = event.event.clone();

            match event.trigger {
                Trigger::Addition => self.beliefs.assert_no_event(ground),
                Trigger::Deletion => self.beliefs.remove_no_event(ground),
            };
        }

        let Some((plan, bindings)) =
            self.plans
                .select(&event, &mut self.plan_selector, &self.beliefs, &self.name)
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

                // "request" perfative runs the literal as an achievement goal directly.
                if performative == Performative::Request {
                    self.handle_event(
                        TriggeringEvent {
                            trigger: Trigger::Addition,
                            event: literal,
                            goal: Some(GoalKind::Achieve),
                        },
                        EventSource::External,
                    );
                    return;
                }

                let literal = Literal {
                    negated: false,
                    structure: Structure {
                        functor: "message".into(),
                        arguments: Some(Box::new([
                            Term::String(performative.as_str().into()),
                            Term::Literal(literal),
                        ])),
                    },
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

    fn handle_messages(&mut self, context: &mut Context<'_, Action>) {
        for _ in 0..self.tick_budget.max_messages.unwrap_or(usize::MAX) {
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
    }
}

impl<State, Action, Perc, Sched, Sel, PSel> BdiAgent<'_, State, Action, Perc, Sched, Sel, PSel>
where
    Perc: Percept,
{
    fn tick_sensors(&mut self, context: &mut Context<'_, Action>) {
        let Some(sensors) = self.sensors.as_mut() else {
            return;
        };

        for _ in 0..self.tick_budget.max_sensors.unwrap_or(sensors.len()) {
            let Some(mut sensor) = sensors.pop_front() else {
                break;
            };

            let Some(percept) = sensor.percept() else {
                sensors.push_back(sensor);
                continue;
            };

            for (trigger, belief) in percept.into_beliefs() {
                let _ = match trigger {
                    Trigger::Addition => self.beliefs.assert(belief, context, None),
                    Trigger::Deletion => self.beliefs.remove(belief, context, None),
                };
            }

            sensors.push_back(sensor);
        }
    }
}

impl<State, Action, Perc, Sched, Sel, PSel> BdiAgent<'_, State, Action, Perc, Sched, Sel, PSel>
where
    Action: Clone,
    Sel: EventSelector,
    PSel: PlanSelector<Action>,
{
    fn handle_events(&mut self) {
        for _ in 0..self.tick_budget.max_events {
            let Some((event, source)) = self.event_queue.next_event() else {
                break;
            };

            self.handle_event(event, source);
        }
    }
}

impl<State, Action, Perc, Sched, Sel, PSel> BdiAgent<'_, State, Action, Perc, Sched, Sel, PSel>
where
    Action: Execute<State = State, UserAction = Action>,
{
    fn run_pending_actions(&mut self, context: &mut Context<'_, Action>) {
        for _ in 0..self
            .tick_budget
            .max_pending_actions
            .unwrap_or(self.pending_actions.len())
        {
            let Some((intention_id, pending)) = self.pending_actions.pop_front() else {
                break;
            };

            match pending.execute(&mut *context, &self.beliefs, &mut self.state) {
                ExecuteResult::Pending(pending) => {
                    self.pending_actions.push_back((intention_id, pending))
                }
                ExecuteResult::Done(Some(_)) => unimplemented!(
                    "Actions that take more than one tick can currently not return any new bindings."
                ),
                ExecuteResult::Done(None) => {
                    if let Some(id) = intention_id {
                        self.intentions.unblock_action(id);
                    }
                }
            }
        }
    }
}

impl<State, Action, Perc, Sched, Sel, PSel> BdiAgent<'_, State, Action, Perc, Sched, Sel, PSel>
where
    Action: Clone + Execute<State = State, UserAction = Action>,
    Sched: Scheduler<Action>,
{
    fn tick_intentions(&mut self, context: &mut Context<'_, Action>) {
        for _ in 0..self.tick_budget.max_intentions {
            if !self.intentions.has_runnable() {
                break;
            }

            let events_before = context.events.len();

            self.intentions
                .step(context, &mut self.beliefs, &mut self.state);

            if !context.actions.is_empty() {
                core::mem::take(&mut context.actions).into_iter().for_each(
                    |(intention, action)| {
                        if let Some(id) = intention {
                            self.intentions.block_on_action(id);
                        }
                        self.pending_actions.push_back((intention, action));
                    },
                );
            }

            // Block any intentions that are newly waiting for an event or action.
            for (source, _) in &context.events[events_before..] {
                if let EventSource::Internal(id) = source {
                    self.intentions.block_on_event(*id);
                }
            }
        }
    }
}

impl<State, Action, Perc, Sched, Sel, PSel> BdiAgent<'_, State, Action, Perc, Sched, Sel, PSel>
where
    Action: Execute<State = State, UserAction = Action> + Clone,
    Perc: Percept,
    Sched: Scheduler<Action>,
    Sel: EventSelector,
    PSel: PlanSelector<Action>,
{
    fn tick(&mut self, environment: &mut Environment) {
        let mut context = Context::new(self.name.clone(), environment);

        self.tick_sensors(&mut context);

        self.handle_messages(&mut context);

        self.handle_events();

        self.run_pending_actions(&mut context);

        self.tick_intentions(&mut context);

        context
            .events
            .into_iter()
            .for_each(|(source, event)| self.event_queue.push(event, source));
    }
}

impl<State, Action, P, Sched, Sel, PSel> Agent for BdiAgent<'_, State, Action, P, Sched, Sel, PSel>
where
    Action: Execute<State = State, UserAction = Action> + Clone,
    P: Percept,
    Sched: Scheduler<Action>,
    Sel: EventSelector,
    PSel: PlanSelector<Action>,
{
    fn update(&mut self, environment: &mut Environment) -> bool {
        match self.fipa.update(environment, &self.name.agent_name) {
            ExecutionState::Initiated => return false,
            ExecutionState::Active => self.tick(environment),
        }
        false
    }

    fn get_name(&self) -> Cow<'_, str> {
        Cow::Borrowed(self.name.agent_name.as_ref())
    }
}

/// Limits on how much work a single [`BdiAgent::update`] tick may perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickBudget {
    /// Maximum number [`ember-bdil`] messages that will be handled. Defaults to `None` (all that are
    /// currently queued).
    pub max_messages: Option<usize>,
    /// Maximum number of internal and external events that be handled. Defaults to 1.
    pub max_events: usize,
    /// Maximum number of sensors polled per tick, round-robin across ticks so no sensor is
    /// starved. Defaults to `None` (every sensor is polled every tick).
    pub max_sensors: Option<usize>,
    /// Maximum number of pending actions repolled per tick, round-robin across ticks.
    /// Defaults to `None` (every pending action is retried every tick).
    pub max_pending_actions: Option<usize>,
    /// Maximum number of intentions stepped per tick. Defaults to `1`.
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

#[cfg(test)]
mod tests {
    use alloc::collections::VecDeque;
    use alloc::string::ToString;
    use alloc::vec;
    use alloc::vec::Vec;

    use crate::bindings::Bindings;
    use crate::knowledge::query::IntoQuery;

    use crate::plan::{Action, BuiltinAction, Formula, ImpureAction};
    use crate::term::owned::composite::VariableOrLiteral;
    use crate::testing::{
        assert_belief, literal, literal_formula, plan, string, trigger, variable, variable_term,
    };
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
        type UserAction = TestAction;

        fn execute<'b>(
            self,
            _bindings: &Bindings<'b>,
            _context: &mut Context<Self::UserAction>,
            _knowledge: &KnowledgeBase,
            state: &mut Self::State,
        ) -> ExecuteResult<'b, Self> {
            match self {
                TestAction::Wait(remaining) => {
                    state.push("poll");
                    if remaining == 0 {
                        ExecuteResult::Done(None)
                    } else {
                        ExecuteResult::Pending(TestAction::Wait(remaining - 1))
                    }
                }
                TestAction::Log(msg) => {
                    state.push(msg);
                    ExecuteResult::Done(None)
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

        // Pinned to `Fifo` so the tick-by-tick assertions below can rely on a deterministic
        // selection order - this test is about blocking semantics, not scheduling.
        let mut agent = BdiAgent::<Vec<&'static str>, TestAction, ()>::new(
            "test-agent",
            Vec::new(),
            None,
            lib,
            vec![literal("wait_test", vec![]), literal("other_test", vec![])],
        )
        .with_intention_scheduler(crate::intention::queue::Fifo);

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
    fn at_does_not_block_the_calling_intention_unlike_wait() {
        // +!start <- .at(3600000, scheduled_goal); .log_after.
        // +!scheduled_goal <- .log_fired.
        let mut lib = PlanLibrary::default();
        lib.add(plan(
            trigger("start", vec![], Some(GoalKind::Achieve)),
            None,
            vec![
                Formula::Action(Action::Builtin(BuiltinAction::at(
                    core::time::Duration::from_secs(3600),
                    literal("scheduled_goal", vec![]),
                ))),
                Formula::Action(Action::User(TestAction::Log("after_at"))),
            ],
        ));
        lib.add(plan(
            trigger("scheduled_goal", vec![], Some(GoalKind::Achieve)),
            None,
            vec![Formula::Action(Action::User(TestAction::Log(
                "scheduled_fired",
            )))],
        ));

        let mut agent = BdiAgent::<Vec<&'static str>, TestAction, ()>::new(
            "test-agent",
            Vec::new(),
            None,
            lib,
            vec![literal("start", vec![])],
        )
        .with_intention_scheduler(crate::intention::queue::Fifo);

        let mut environment = new_environment();

        // Tick 1: `.at`'s first poll re-queues itself non-blocking and completes immediately.
        agent.tick(&mut environment);
        assert_eq!(agent.pending_actions.len(), 1);
        assert!(agent.intentions.has_runnable());

        // Tick 2: the same intention is free to run its next formula straight away, even
        // though the re-queued `.at` is still sitting in `pending_actions` an hour from firing.
        agent.tick(&mut environment);
        assert_eq!(agent.state, vec!["after_at"]);
        assert_eq!(agent.pending_actions.len(), 1, "still waiting on its delay");
        assert!(!agent.state.contains(&"scheduled_fired"));
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
                Formula::Action(Action::Builtin(BuiltinAction::Impure(
                    ImpureAction::Forall {
                        query: literal_formula("item", vec![variable_term(&x)]),
                        goal: VariableOrLiteral::Literal(literal(
                            "mark_processed",
                            vec![variable_term(&x)],
                        )),
                    },
                ))),
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
                silent: false,
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
            let mut query = (&query_formula).into_query(&agent.beliefs, &agent.name);
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
        type UserAction = RecordArg;

        fn execute<'b>(
            self,
            bindings: &Bindings<'b>,
            _context: &mut Context<Self::UserAction>,
            _knowledge: &KnowledgeBase,
            state: &mut Self::State,
        ) -> ExecuteResult<'b, Self> {
            let seen = bindings
                .lookup_as_type::<alloc::string::String>(&self.0)
                .and_then(Result::ok)
                .unwrap_or_else(|| "<unbound>".into());
            state.push(seen);
            ExecuteResult::Done(None)
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
        // run the action, one more to notice the body is now empty), and the scheduler keeps
        // re-selecting an intention until it's actually removed - so budget generously rather
        // than trying to hit an exact step count.
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
                (
                    Trigger::Addition,
                    literal("ping", vec![crate::testing::number(1.0)]),
                ),
                (
                    Trigger::Addition,
                    literal("ping", vec![crate::testing::number(2.0)]),
                ),
                (
                    Trigger::Addition,
                    literal("ping", vec![crate::testing::number(3.0)]),
                ),
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

        let mut agent = BdiAgent::<Vec<&'static str>, TestAction, Burst>::new(
            "event-budget-agent",
            Vec::new(),
            None,
            lib,
            vec![],
        );
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
    fn test_request_performative_fires_the_goal_directly_with_no_catch_plan() {
        // A Request-performative message is only ever built directly in Rust, never by ASL `.send(...)`.
        let mut lib = PlanLibrary::default();
        lib.add(plan(
            trigger("do_thing", vec![], Some(GoalKind::Achieve)),
            None,
            vec![Formula::Action(Action::User(TestAction::Log("fired")))],
        ));
        // No `+message(...)` plan here on purpose, catches a regression to the usual wrap.

        let mut agent = BdiAgent::<Vec<&'static str>, TestAction, ()>::new(
            "request-agent",
            Vec::new(),
            None,
            lib,
            vec![],
        );

        let message = Message {
            performative: Performative::Request,
            receiver: None,
            ontology: None,
            other: None,
            content: Some(Content::Bdil(BdilContent::Literal(
                literal("do_thing", vec![]).into(),
            ))),
        };
        let mut environment = Environment::new(VecDeque::from([message]));

        agent.tick(&mut environment);

        assert_eq!(agent.state, vec!["fired"]);
    }

    #[test]
    fn test_self_recursive_goal_does_not_leak_stack_frames() {
        // `+!loop <- !loop.`, the standard AgentSpeak "run forever" idiom used throughout the
        // microgrid/traffic-light case studies (heartbeat_loop, led_supervisor). Each recursion
        // resolves the plan body down to nothing but re-raises the same internal event, which
        // must replace the frame it came from, not stack a dead one underneath it.
        let mut lib = PlanLibrary::<TestAction>::default();
        lib.add(plan(
            trigger("loop", vec![], Some(GoalKind::Achieve)),
            None,
            vec![Formula::Goal {
                kind: GoalKind::Achieve,
                goal: literal("loop", Vec::with_capacity(0)),
            }],
        ));

        let agent = BdiAgent::<Vec<&'static str>, TestAction, ()>::new(
            "loop-agent",
            Vec::new(),
            None,
            lib,
            vec![literal("loop", Vec::with_capacity(0))],
        );
        let mut agent = agent.with_tick_budget(TickBudget {
            max_events: 10,
            max_intentions: 10,
            ..TickBudget::default()
        });

        let mut environment = new_environment();
        for _ in 0..50 {
            agent.tick(&mut environment);
            // Momentarily absent (between the old frame's own recursion emitting the next
            // `!loop` and that event being handled) is fine; more than one frame stacked up
            // for it is the leak this guards against.
            if let Some(len) = agent.intentions.intention_stack_len(0) {
                assert_eq!(
                    len, 1,
                    "self-recursive goal must not accumulate dead frames underneath the live one"
                );
            }
        }
    }

    #[test]
    fn test_custom_plan_selector_replaces_first_applicable() {
        // Reject any applicable plan with a one-step body; among these two same-triggered
        // plans, that rejects the first (added) one and forces the second to be selected -
        // `FirstApplicable` would have picked the first instead.
        struct RejectShortBody;
        impl<A> crate::plan::selector::PlanSelector<A> for RejectShortBody {
            fn filter_plan<'p>(
                &mut self,
                plan: &'p crate::plan::Plan<A>,
            ) -> Option<&'p crate::plan::Plan<A>> {
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

        let mut agent = BdiAgent::<Vec<&'static str>, TestAction, (), Random, FirstEvent>::new(
            "selector-agent",
            Vec::new(),
            None,
            lib,
            vec![literal("start", vec![])],
        )
        .with_plan_selector(RejectShortBody);

        let mut environment = new_environment();
        for _ in 0..10 {
            if agent.intentions.is_empty() {
                break;
            }
            agent.tick(&mut environment);
        }

        assert_eq!(agent.state, vec!["second", "marker"]);
    }

    /// Regression test for ChirpPark bug 4b -- distinct from 0952b00 (stale
    /// answer under backtracking within one query); this one dropped the
    /// caller's bindings in a single fresh query, no backtracking involved.
    #[test]
    fn test_gateway_retry_across_separate_invocations() {
        use crate::knowledge::belief::Knowledge;
        use crate::plan::{
            ArithmeticExpression, ArithmeticOperator, CompareOperator, LogicalOperator,
            QueryFormula, RelationalOperator, RelationalQueryFormula,
        };
        use crate::testing::number;

        fn rule(functor: &str, args: Vec<Term>, body: QueryFormula) -> Knowledge {
            let lit = literal(functor, args);
            (lit, body).into()
        }

        fn and(ops: Vec<QueryFormula>) -> QueryFormula {
            QueryFormula::Logical {
                operator: LogicalOperator::Conjunction,
                operands: ops.into_boxed_slice(),
            }
        }

        fn not(op: QueryFormula) -> QueryFormula {
            QueryFormula::Not(Box::new(op))
        }

        fn expr(t: Term) -> ArithmeticExpression {
            ArithmeticExpression::Term(t)
        }

        fn minus(a: ArithmeticExpression, b: ArithmeticExpression) -> ArithmeticExpression {
            ArithmeticExpression::Operation {
                operator: ArithmeticOperator::Min,
                operands: vec![a, b].into_boxed_slice(),
            }
        }

        fn gt(l: ArithmeticExpression, r: ArithmeticExpression) -> QueryFormula {
            QueryFormula::Relational(RelationalQueryFormula {
                operator: RelationalOperator::Compare {
                    operator: CompareOperator::GreaterThan,
                    equal: false,
                },
                operands: (l, r),
            })
        }

        // recent_reply(GW) :- last_reply(GW, T) & now(Now) & not Now - T > 5000.0.
        // gateway_down(GW) :- last_request(GW, ReqT) & now(Now) & not recent_reply(GW)
        //                      & Now - ReqT > 5000.0.
        // Exactly va.rs's two rules, verbatim in shape.
        let gw1 = variable();
        let treply = variable();
        let now2 = variable();
        let recent_reply = rule(
            "recent_reply",
            vec![variable_term(&gw1)],
            and(vec![
                literal_formula(
                    "last_reply",
                    vec![variable_term(&gw1), variable_term(&treply)],
                ),
                literal_formula("now", vec![variable_term(&now2)]),
                not(gt(
                    minus(expr(variable_term(&now2)), expr(variable_term(&treply))),
                    expr(number(5000.0)),
                )),
            ]),
        );

        let gw2 = variable();
        let treq = variable();
        let now3 = variable();
        let gateway_down = rule(
            "gateway_down",
            vec![variable_term(&gw2)],
            and(vec![
                literal_formula(
                    "last_request",
                    vec![variable_term(&gw2), variable_term(&treq)],
                ),
                literal_formula("now", vec![variable_term(&now3)]),
                not(literal_formula("recent_reply", vec![variable_term(&gw2)])),
                gt(
                    minus(expr(variable_term(&now3)), expr(variable_term(&treq))),
                    expr(number(5000.0)),
                ),
            ]),
        );

        let mut beliefs = KnowledgeBase::default();
        beliefs.assert_no_event(recent_reply);
        beliefs.assert_no_event(gateway_down);
        assert_belief(&mut beliefs, "gateway_a", vec![string("ga-1")]);
        assert_belief(&mut beliefs, "gateway_b", vec![string("ga-2")]);
        assert_belief(&mut beliefs, "now", vec![number(0.0)]);

        // +!select_gateway : gateway_a(GW) & not gateway_down(GW) <- +target_gateway(GW).
        // +!select_gateway : gateway_b(GW) & not gateway_down(GW) <- +target_gateway(GW).
        // Same "one plan per candidate" shape va.rs already uses to sidestep the
        // *within-query* backtracking bug (0952b00's own target).
        let mut lib = PlanLibrary::<TestAction>::default();
        let gwa = variable();
        lib.add(plan(
            trigger("select_gateway", vec![], Some(GoalKind::Achieve)),
            Some(and(vec![
                literal_formula("gateway_a", vec![variable_term(&gwa)]),
                not(literal_formula("gateway_down", vec![variable_term(&gwa)])),
            ])),
            vec![Formula::Belief {
                trigger: Trigger::Addition,
                belief: literal("target_gateway", vec![variable_term(&gwa)]),
                silent: false,
            }],
        ));
        let gwb = variable();
        lib.add(plan(
            trigger("select_gateway", vec![], Some(GoalKind::Achieve)),
            Some(and(vec![
                literal_formula("gateway_b", vec![variable_term(&gwb)]),
                not(literal_formula("gateway_down", vec![variable_term(&gwb)])),
            ])),
            vec![Formula::Belief {
                trigger: Trigger::Addition,
                belief: literal("target_gateway", vec![variable_term(&gwb)]),
                silent: false,
            }],
        ));

        let mut agent = BdiAgent::<Vec<&'static str>, TestAction, ()>::new(
            "va-poc",
            Vec::new(),
            Some(beliefs),
            lib,
            vec![],
        );
        let mut environment = new_environment();

        // --- Invocation 1: cold boot, neither gateway has ever been contacted. ---
        agent.handle_event(
            trigger("select_gateway", vec![], Some(GoalKind::Achieve)),
            EventSource::External,
        );
        for _ in 0..5 {
            agent.tick(&mut environment);
        }

        let ga1 = literal_formula("target_gateway", vec![string("ga-1")]);
        let ga2 = literal_formula("target_gateway", vec![string("ga-2")]);
        assert!(
            (&ga1)
                .into_query(&agent.beliefs, &agent.name)
                .next_bindings(None)
                .is_some(),
            "invocation 1: cold boot should target ga-1 (checked first, neither gateway down yet)"
        );

        // --- Between invocations: exactly what va.rs's own plans do in the real
        // timeline -- a request went to ga-1, it never replied, now() advances
        // past the 5s timeout, so the periodic `now(T)` plan retracts
        // target_gateway and re-fires `!select_gateway`. ga-2 was never
        // contacted at all. ---
        agent
            .beliefs
            .remove_no_event(literal("target_gateway", vec![string("ga-1")]));
        agent
            .beliefs
            .assert_no_event(literal("last_request", vec![string("ga-1"), number(0.0)]));
        agent
            .beliefs
            .remove_no_event(literal("now", vec![number(0.0)]));
        agent
            .beliefs
            .assert_no_event(literal("now", vec![number(6000.0)]));

        // --- Invocation 2: a separate, later `!select_gateway` re-invocation,
        // not a backtrack within the same query -- ga-1 is genuinely down,
        // ga-2 genuinely is not (no last_request(ga-2, _) exists at all). ---
        agent.handle_event(
            trigger("select_gateway", vec![], Some(GoalKind::Achieve)),
            EventSource::External,
        );
        for _ in 0..5 {
            agent.tick(&mut environment);
        }

        // --- Isolation check: does plan B's *exact* context formula, evaluated
        // the exact same way `ApplicablePlanSelection::next_plan` evaluates it
        // (fresh Query, `Some(&Bindings::empty())` as `existing_bindings`, not
        // `None`), succeed when run directly, bypassing plan/event/intention
        // scheduling entirely? If yes, the bug is *not* in query evaluation at
        // all -- it's in how plan selection got invoked the second time.
        let gwb2 = variable();
        let plan_b_context_copy = and(vec![
            literal_formula("gateway_b", vec![variable_term(&gwb2)]),
            not(literal_formula("gateway_down", vec![variable_term(&gwb2)])),
        ]);
        let empty_bindings = crate::bindings::Bindings::empty();
        let isolated_result = (&plan_b_context_copy)
            .into_query(&agent.beliefs, &agent.name)
            .next_bindings(Some(&empty_bindings));

        // Same conjunction, but with `None` instead of `Some(&Bindings::empty())` --
        // isolates whether the `Some`/`None` distinction itself matters.
        let gwb3 = variable();
        let plan_b_context_copy2 = and(vec![
            literal_formula("gateway_b", vec![variable_term(&gwb3)]),
            not(literal_formula("gateway_down", vec![variable_term(&gwb3)])),
        ]);
        let isolated_result_none = (&plan_b_context_copy2)
            .into_query(&agent.beliefs, &agent.name)
            .next_bindings(None);

        // Same conjunction, but with the gateway hardcoded as a ground string
        // instead of a variable bound via `gateway_b(GW)` -- isolates whether
        // going through a *bound variable* (vs. a literal ground term) into the
        // negated nested-rule lookup is what's different.
        let ground_conjunction = and(vec![
            literal_formula("gateway_b", vec![string("ga-2")]),
            not(literal_formula("gateway_down", vec![string("ga-2")])),
        ]);
        let ground_result = (&ground_conjunction)
            .into_query(&agent.beliefs, &agent.name)
            .next_bindings(Some(&empty_bindings));

        // Sharpest isolation: evaluate *only* `not gateway_down(GW)` on its own,
        // with GW pre-bound to "ga-2" via a real `Bindings` map (not threaded
        // through a preceding conjunct at all). If this still reports
        // gateway_down as true, the rule's own internal resolution is ignoring
        // the caller's binding for its argument entirely, rather than being a
        // caching/backtracking artifact.
        let gwb4 = variable();
        let ga2_term = string("ga-2");
        let pre_bound = crate::testing::bindings(vec![(
            gwb4.clone(),
            crate::term::view::TermView::Term(&ga2_term),
        )]);
        let solo_negation = not(literal_formula("gateway_down", vec![variable_term(&gwb4)]));
        let solo_result = (&solo_negation)
            .into_query(&agent.beliefs, &agent.name)
            .next_bindings(Some(&pre_bound));

        // --- Diagnostics: what actually happened? ---
        let gd_a = literal_formula("gateway_down", vec![string("ga-1")]);
        let gd_b = literal_formula("gateway_down", vec![string("ga-2")]);
        let debug = alloc::format!(
            "target_gateway(ga-1)={} target_gateway(ga-2)={} gateway_down(ga-1)={} gateway_down(ga-2)={} \
             isolated_plan_b_context(Some(empty))={} isolated_plan_b_context(None)={} \
             ground_conjunction(Some(empty))={} solo_negation_with_GW_prebound_to_ga-2={}",
            (&ga1)
                .into_query(&agent.beliefs, &agent.name)
                .next_bindings(None)
                .is_some(),
            (&ga2)
                .into_query(&agent.beliefs, &agent.name)
                .next_bindings(None)
                .is_some(),
            (&gd_a)
                .into_query(&agent.beliefs, &agent.name)
                .next_bindings(None)
                .is_some(),
            (&gd_b)
                .into_query(&agent.beliefs, &agent.name)
                .next_bindings(None)
                .is_some(),
            isolated_result.is_some(),
            isolated_result_none.is_some(),
            ground_result.is_some(),
            solo_result.is_some(),
        );

        assert!(
            (&ga2)
                .into_query(&agent.beliefs, &agent.name)
                .next_bindings(None)
                .is_some(),
            "invocation 2: ga-1 is down and ga-2 is not -- select_gateway should retarget \
             to ga-2. If this fails, the retry is stuck exactly like the real VA. {debug}"
        );
    }
}
