use alloc::borrow::Cow;

use ember_core::agent::Agent as EmberAgent;
use ember_core::context::ContainerContext;

use crate::context::Context;
use crate::event::EventSource;
use crate::event::queue::EventQueue;
use crate::event::selector::FirstEvent;
use crate::intention::queue::{Fifo, IntentionQueue};
use crate::knowledge::store::BeliefBase;
use crate::plan::library::PlanLibrary;
use crate::plan::selector::FirstApplicable;
use crate::plan::{Trigger, TriggeringEvent};
use crate::sensor::Sensor;

#[derive(Debug)]
pub struct BdiAgent<'s, const S: usize, Agent, Action, Percept> {
    name: Cow<'static, str>,
    agent: Agent,
    beliefs: BeliefBase,
    plans: PlanLibrary<Action>,
    intentions: IntentionQueue<Action>,
    event_queue: EventQueue,
    sensors: [Sensor<'s, Percept>; S],
}

impl<'s, Agent, const S: usize, Action, Percept> BdiAgent<'s, S, Agent, Action, Percept>
where
    Action: Clone,
{
    pub fn new(
        name: impl Into<Cow<'static, str>>,
        agent: Agent,
        sensors: [Sensor<'s, Percept>; S],
        beliefs: Option<BeliefBase>,
        plans: PlanLibrary<Action>,
        initial_goals: impl IntoIterator<Item = TriggeringEvent>,
    ) -> Self {
        let mut this = Self {
            name: name.into(),
            agent,
            beliefs: beliefs.unwrap_or_default(),
            plans,
            intentions: IntentionQueue::default(),
            event_queue: EventQueue::default(),
            sensors,
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
                Trigger::Addition => self.beliefs.assert(ground),
                Trigger::Deletion => self.beliefs.remove(ground),
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

impl<A, const S: usize, Action, Percept> EmberAgent for BdiAgent<'_, S, A, Action, Percept>
where
    A: Agent<Action = Action, Percept = Percept>,
    Action: Clone,
{
    fn update(&mut self, _context: &mut ContainerContext) -> bool {
        // TODO: Implement interaction with the ember framework.

        for sensor in self.sensors.iter_mut() {
            let Some(percept) = sensor.percept() else {
                continue;
            };
            self.agent.handle_percept(percept, &mut self.beliefs);
        }

        if let Some((event, source)) = self.event_queue.next_event(FirstEvent) {
            self.handle_event(event, source);
        }

        let mut context = Context::new();
        self.intentions.step(&mut Fifo, &mut context);

        while let Some(action) = context.actions.pop() {
            self.agent.perform_action(action, &mut context);
        }

        context.events.into_iter().for_each(|(source, event)| {
            self.event_queue.push(event, source);
        });

        self.intentions.is_empty()
    }

    fn get_name(&self) -> Cow<str> {
        self.name.clone()
    }
}

pub trait Agent {
    type Action;
    type Percept;

    fn perform_action(&mut self, action: Self::Action, context: &mut Context<Self::Action>);

    fn handle_percept(&mut self, percept: Self::Percept, knowledge: &mut BeliefBase);
}
