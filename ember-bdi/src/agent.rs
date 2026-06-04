use alloc::borrow::Cow;

use alloc::boxed::Box;
use ember_core::agent::Agent;
use ember_core::environment::Environment;

use crate::context::Context;
use crate::event::EventSource;
use crate::event::queue::EventQueue;
use crate::event::selector::FirstEvent;
use crate::intention::queue::{Fifo, IntentionQueue};
use crate::knowledge::store::BeliefBase;
use crate::plan::action::Execute;
use crate::plan::library::PlanLibrary;
use crate::plan::selector::FirstApplicable;
use crate::plan::{Trigger, TriggeringEvent};
use crate::sensor::{Percept, Sensor};

#[derive(Debug)]
pub struct BdiAgent<'s, State, Action, Percept> {
    name: Cow<'static, str>,
    state: State,
    beliefs: BeliefBase,
    plans: PlanLibrary<Action>,
    intentions: IntentionQueue<Action>,
    event_queue: EventQueue,
    sensors: Box<[Sensor<'s, Percept>]>,
}

impl<'s, State, Action, Percept> BdiAgent<'s, State, Action, Percept>
where
    Action: Clone,
{
    pub fn new(
        name: impl Into<Cow<'static, str>>,
        state: State,
        sensors: impl Into<Box<[Sensor<'s, Percept>]>>,
        beliefs: Option<BeliefBase>,
        plans: PlanLibrary<Action>,
        initial_goals: impl IntoIterator<Item = TriggeringEvent>,
    ) -> Self {
        let mut this = Self {
            name: name.into(),
            state,
            beliefs: beliefs.unwrap_or_default(),
            plans,
            intentions: IntentionQueue::default(),
            event_queue: EventQueue::default(),
            sensors: sensors.into(),
        };
        initial_goals
            .into_iter()
            .for_each(|g| this.handle_event(g, EventSource::External));
        this
    }

    fn handle_event(&mut self, event: TriggeringEvent, source: EventSource) {
        if event.goal.is_none() {
            let ground = event
                .event
                .clone()
                .try_into_ground()
                .expect("belief update must be ground");

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
        self.intentions.push(plan, bindings, existing_intention);
    }
}

impl<State, Action, P> Agent for BdiAgent<'_, State, Action, P>
where
    Action: Execute<State = State, Action = Action> + Clone,
    P: Percept,
{
    fn update(&mut self, environment: &mut Environment) -> bool {
        let mut context = Context::new(environment);

        for sensor in self.sensors.iter_mut() {
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

        if let Some((event, source)) = self.event_queue.next_event(FirstEvent) {
            self.handle_event(event, source);
        }

        let bindings = self.intentions.step(&mut Fifo, &mut context);

        while let Some(action) = context.actions.pop() {
            use crate::plan::Action::*;
            match action {
                Builtin(action) => action.execute(&bindings, &mut context),
                User(action) => action.execute(&bindings, &mut context, &mut self.state),
            }
        }

        context.events.into_iter().for_each(|(source, event)| {
            self.event_queue.push(event, source);
        });

        drop(bindings);
        self.intentions.is_empty()
    }

    fn get_name(&self) -> Cow<str> {
        self.name.clone()
    }
}
