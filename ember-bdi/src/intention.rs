use alloc::vec::Vec;

use derive_where::derive_where;

use crate::bindings::{Bindings, OwnedBindings};
use crate::context::Context;
use crate::knowledge::base::KnowledgeBase;
use crate::plan::{Formula, Plan, Trigger, TriggeringEvent};

use self::result::*;

pub mod queue;
pub(crate) mod result;

pub use self::queue::{Fifo, Scheduler};

pub type IntentionId = usize;

#[derive_where(Debug)]
pub struct Intention<A> {
    id: IntentionId,
    stack: Vec<Frame<A>>,
}

impl<A> Intention<A> {
    pub(crate) fn step(
        &mut self,
        context: &mut Context<A>,
        knowledge: &mut KnowledgeBase,
    ) -> Result {
        let Some(frame) = self.stack.last_mut() else {
            return StepOk::done();
        };

        let bindings = match frame.step(context, knowledge)? {
            StepOk::Done => frame.take_filtered_bindings(),
            StepOk::Pending => return StepOk::pending(),
        };

        self.stack.pop(); // Remove the done frame

        let Some(next_frame) = self.stack.last_mut() else {
            return StepOk::done();
        };

        next_frame.bindings = Bindings::merge([
            bindings,
            core::mem::replace(&mut next_frame.bindings, Bindings::empty()),
        ])
        .expect("merging bindings between frames failed");

        StepOk::pending()
    }

    pub(crate) fn get_last_bindings(&self) -> Option<&OwnedBindings> {
        Some(&self.stack.last()?.bindings)
    }

    pub(crate) fn take_last_bindings(&mut self) -> OwnedBindings {
        self.stack
            .last_mut()
            .map(|f| f.take_filtered_bindings())
            .unwrap_or_else(OwnedBindings::empty)
    }
}

impl<A> Intention<A> {
    fn new(id: IntentionId) -> Self {
        Self {
            id,
            stack: Vec::default(),
        }
    }
}

impl<A: Clone> Intention<A> {
    pub(crate) fn push(
        &mut self,
        plan: &'_ Plan<A>,
        bindings: Bindings<'_>,
        event: TriggeringEvent,
    ) {
        self.stack.push(Frame::new(plan, bindings, self.id, event))
    }
}

#[derive_where(Debug)]
struct Frame<A> {
    /// The id of the intention this frame belongs to.
    intention_id: IntentionId,
    /// The event that triggered the creation of this frame. Used to stop the execution of
    /// plans, and to filter bindings.
    event: TriggeringEvent,
    /// Bindings that this frame is created with and that have been resolved during the
    /// execution of this frame.
    bindings: OwnedBindings,
    /// Remaining parts of the plan body to execute.
    remaining: Vec<Formula<A>>,
}

impl<A: Clone> Frame<A> {
    fn new(
        plan: &'_ Plan<A>,
        bindings: Bindings<'_>,
        intention_id: IntentionId,
        event: TriggeringEvent,
    ) -> Self {
        Self {
            intention_id,
            event,
            bindings: bindings.into(),
            remaining: plan.body.iter().rev().cloned().collect(),
        }
    }
}

impl<A> Frame<A> {
    fn step(&mut self, context: &mut Context<A>, knowledge: &mut KnowledgeBase) -> Result {
        let Some(formula) = self.remaining.pop() else {
            return StepOk::done();
        };

        let formula = formula.resolve_possible(&self.bindings)?;

        match formula {
            Formula::Belief {
                trigger,
                belief,
                silent,
            } => {
                let event = if !belief.is_ground() {
                    return Err(StepError::ResolveIncomplete);
                } else {
                    belief
                };
                match trigger {
                    Trigger::Addition => {
                        if silent {
                            knowledge.assert_no_event(event);
                        } else {
                            knowledge.assert(event, context, Some(self.intention_id));
                        }
                    }
                    Trigger::Deletion => {
                        if silent {
                            knowledge.remove_no_event(event);
                        } else {
                            knowledge.remove(event, context, Some(self.intention_id));
                        }
                    }
                }
            }
            Formula::Goal { kind, goal } => context.emit_event(
                TriggeringEvent {
                    trigger: Trigger::Addition,
                    event: goal,
                    goal: Some(kind),
                },
                Some(self.intention_id),
            ),
            Formula::Action(action) => context.perform_action(self.intention_id, action),
        }

        StepOk::pending()
    }

    fn take_filtered_bindings(&mut self) -> OwnedBindings {
        let vars = self.event.event.variables();
        let mut map = alloc::collections::BTreeMap::new();

        if let Some(bindings) = &self.bindings.bindings {
            for v_id in vars {
                if let Some(val) = bindings.get(&v_id) {
                    map.insert(v_id, val.clone());
                }
            }
        }

        OwnedBindings::new(map, crate::bindings::AliasMap::empty())
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use crate::bindings::Bindings;
    use crate::plan::{Action, Formula, Trigger};

    use crate::testing::*;

    use super::*;

    #[test]
    fn test_intention_step_empty() {
        let mut intention: Intention<()> = Intention::new(0);
        // SAFETY: The environment on the context remains untouched,
        let mut context = unsafe { new_context_without_environment() };
        let mut knowledge = KnowledgeBase::default();

        // Step with no frames returns Done
        assert!(matches!(
            intention.step(&mut context, &mut knowledge),
            Ok(StepOk::Done)
        ));
    }

    #[test]
    fn test_intention_push_and_step() {
        let mut intention: Intention<()> = Intention::new(0);
        // SAFETY: The environment on the context remains untouched,
        let mut context = unsafe { new_context_without_environment() };
        let mut knowledge = KnowledgeBase::default();

        let trigger = trigger("event", vec![], None);
        let plan = plan(trigger.clone(), None, vec![]);

        intention.push(&plan, Bindings::empty(), trigger);

        assert_eq!(intention.stack.len(), 1);

        // Plan has no body, so one step should complete the frame, merge bindings, and remove the frame.
        // It returns Done because the intention has no more frames.
        let result = intention.step(&mut context, &mut knowledge);
        assert!(matches!(result, Ok(StepOk::Done)));
        assert_eq!(intention.stack.len(), 0);
    }

    #[test]
    fn test_intention_step_with_actions() {
        let mut intention: Intention<&'static str> = Intention::new(0);
        // SAFETY: The environment on the context remains untouched,
        let mut context = unsafe { new_context_without_environment() };
        let mut knowledge = KnowledgeBase::default();

        let trigger = trigger("event", vec![], None);
        let plan = plan(
            trigger.clone(),
            None,
            vec![
                Formula::Action(Action::User("action1")),
                Formula::Action(Action::User("action2")),
            ],
        );

        intention.push(&plan, Bindings::empty(), trigger);

        // step 1: executes action1 (because it's popped first)
        let result = intention.step(&mut context, &mut knowledge);
        assert!(matches!(result, Ok(StepOk::Pending)));
        assert_eq!(context.actions, &[(intention.id, Action::User("action1"))]);

        // step 2: executes action2
        let result = intention.step(&mut context, &mut knowledge);
        assert!(matches!(result, Ok(StepOk::Pending)));
        assert_eq!(
            context.actions,
            &[
                (intention.id, Action::User("action1")),
                (intention.id, Action::User("action2"))
            ]
        );

        // step 3: frame done, intention done
        let result = intention.step(&mut context, &mut knowledge);
        assert!(matches!(result, Ok(StepOk::Done)));
    }

    #[test]
    fn test_intention_step_with_beliefs_and_goals() {
        let mut intention: Intention<()> = Intention::new(0);
        // SAFETY: The environment on the context remains untouched,
        let mut context = unsafe { new_context_without_environment() };
        let mut knowledge = KnowledgeBase::default();

        let trigger = trigger("event", vec![], None);
        let plan = plan(
            trigger.clone(),
            None,
            vec![
                Formula::Goal {
                    kind: crate::plan::GoalKind::Achieve,
                    goal: literal("goal1", Vec::with_capacity(0)),
                },
                Formula::Belief {
                    trigger: Trigger::Addition,
                    belief: literal("belief1", Vec::with_capacity(0)),
                    silent: false,
                },
            ],
        );

        intention.push(&plan, Bindings::empty(), trigger);

        let result = intention.step(&mut context, &mut knowledge);
        assert!(matches!(result, Ok(StepOk::Pending)));

        let result = intention.step(&mut context, &mut knowledge);
        assert!(matches!(result, Ok(StepOk::Pending)));
    }

    // Regression test for the microgrid route-churn bug (see
    // `ember-case-studies/microgrid-route-churn-bug.md`): a plan body's own `+belief`/`-belief`
    // must be queryable immediately, in the same step that executes it, not several ticks later
    // once its emitted event happens to reach the front of the outer event queue. Without the
    // synchronous `assert_no_event`/`remove_no_event` call in `Frame::step`'s `Formula::Belief`
    // arm, this belief would only become visible after a later, separate `handle_event` call.
    #[test]
    fn belief_formula_is_queryable_immediately_after_its_own_step() {
        use crate::knowledge::query::IntoQuery;

        let mut intention: Intention<()> = Intention::new(0);
        // SAFETY: The environment on the context remains untouched.
        let mut context = unsafe { new_context_without_environment() };
        let mut knowledge = KnowledgeBase::default();

        let trigger = trigger("event", vec![], None);
        let plan = plan(
            trigger.clone(),
            None,
            vec![Formula::Belief {
                trigger: Trigger::Addition,
                belief: literal("route_active", vec![string("load"), string("c")]),
                silent: false,
            }],
        );

        intention.push(&plan, Bindings::empty(), trigger);

        // A single step executes the belief formula (and only that -- the frame isn't done yet,
        // nothing has drained the emitted event through `handle_event`).
        let result = intention.step(&mut context, &mut knowledge);
        assert!(matches!(result, Ok(StepOk::Pending)));

        let query_formula = literal_formula("route_active", vec![string("load"), string("c")]);
        assert!(
            (&query_formula)
                .into_query(&knowledge)
                .next_bindings(None)
                .is_some(),
            "belief should already be queryable right after the step that added it"
        );
    }
}
