use alloc::borrow::Cow;

use alloc::boxed::Box;
use ember_core::agent::Agent as EmberAgent;
use ember_core::context::ContainerContext;

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
pub struct BdiAgent<'s, Agent, Action, Percept> {
    name: Cow<'static, str>,
    agent: Agent,
    beliefs: BeliefBase,
    plans: PlanLibrary<Action>,
    intentions: IntentionQueue<Action>,
    event_queue: EventQueue,
    sensors: Box<[Sensor<'s, Percept>]>,
}

impl<'s, Agent, Action, Percept> BdiAgent<'s, Agent, Action, Percept>
where
    Action: Clone,
{
    pub fn new(
        name: impl Into<Cow<'static, str>>,
        agent: Agent,
        sensors: impl Into<Box<[Sensor<'s, Percept>]>>,
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

impl<Agent, Action, P> EmberAgent for BdiAgent<'_, Agent, Action, P>
where
    Action: Execute<Agent = Agent, Action = Action> + Clone,
    P: Percept,
{
    fn update(&mut self, _context: &mut ContainerContext) -> bool {
        // TODO: Implement interaction with the ember framework.
        //
        let mut context = Context::new();

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
                User(action) => action.execute(&bindings, &mut context, &mut self.agent),
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
