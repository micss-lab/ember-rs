use alloc::borrow::Cow;
use alloc::vec::Vec;

use ember_core::agent::Agent;
use ember_core::environment::Environment;

use crate::context::Context;
use crate::event::EventSource;
use crate::event::queue::EventQueue;
use crate::event::selector::FirstEvent;
use crate::intention::queue::{Fifo, IntentionQueue};
use crate::knowledge::store::BeliefBase;
use crate::literal::Literal;
use crate::plan::action::Execute;
use crate::plan::library::PlanLibrary;
use crate::plan::selector::FirstApplicable;
use crate::plan::{GoalKind, Trigger, TriggeringEvent};
use crate::sensor::{Percept, Perceptor, Sensor};

#[derive(Debug)]
pub struct BdiAgent<'s, State, Action, Percept> {
    name: Cow<'static, str>,
    state: State,
    beliefs: BeliefBase,
    plans: PlanLibrary<Action>,
    intentions: IntentionQueue<Action>,
    event_queue: EventQueue,
    sensors: Option<Vec<Sensor<'s, Percept>>>,
}

impl<'s, State, Action, Percept> BdiAgent<'s, State, Action, Percept>
where
    Action: Clone,
{
    pub fn new(
        name: impl Into<Cow<'static, str>>,
        state: State,
        beliefs: Option<BeliefBase>,
        plans: PlanLibrary<Action>,
        initial_goals: impl IntoIterator<Item = Literal>,
    ) -> Self {
        let mut this = Self {
            name: name.into(),
            state,
            beliefs: beliefs.unwrap_or_default(),
            plans,
            intentions: IntentionQueue::default(),
            event_queue: EventQueue::default(),
            sensors: None,
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

impl<State, Action, P> Agent for BdiAgent<'_, State, Action, P>
where
    Action: Execute<State = State, Action = Action> + Clone,
    P: Percept,
{
    fn update(&mut self, environment: &mut Environment) -> bool {
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
