use alloc::string::ToString;
use alloc::vec::Vec;

use derive_where::derive_where;

use crate::bindings::resolver::ResolveFailure;
use crate::bindings::{Bindings, OwnedBindings};
use crate::context::Context;
use crate::plan::{Action, Formula, Plan, Trigger, TriggeringEvent};

pub(crate) mod queue;

pub(crate) type IntentionId = usize;

#[derive_where(Default)]
#[derive(Debug)]
pub(crate) struct Intention<A> {
    id: IntentionId,
    stack: Vec<Frame<A>>,
}

impl<A> Intention<A> {
    pub(crate) fn step(&mut self, context: &mut Context<A>) -> IntentionRunResult {
        let Some(frame) = self.stack.last_mut() else {
            return IntentionRunResult::Done;
        };

        let bindings = match frame.step(context) {
            Ok(FrameStep::Done { bindings }) => bindings,
            Ok(FrameStep::NotDone) => return IntentionRunResult::NotDone,
            Err(_) => return IntentionRunResult::Done,
        };

        self.stack.pop();

        let Some(frame) = self.stack.last_mut() else {
            return IntentionRunResult::Done;
        };

        frame.bindings = Bindings::merge([
            core::mem::replace(&mut frame.bindings, Bindings::empty()),
            bindings,
        ])
        .expect("merging bindings between frames failed");
        IntentionRunResult::NotDone
    }
}

impl<A: Clone> Intention<A> {
    fn push(&mut self, plan: &'_ Plan<A>, bindings: Bindings<'_>) {
        self.stack.push(Frame::new(plan, bindings, self.id))
    }
}

pub(crate) enum IntentionRunResult {
    NotDone,
    Done,
}

#[derive(Debug)]
struct Frame<A> {
    /// The id of the intention this frame belongs to.
    intention_id: IntentionId,
    /// The event that triggered the creation of this frame. Used to stop the execution of
    /// plans.
    _event: TriggeringEvent,
    /// Bindings that this frame is created with and that have been resolved during the
    /// execution of this frame.
    bindings: OwnedBindings,
    /// Remaining parts of the plan body to execute.
    remaining: Vec<Formula<A>>,
}

impl<A: Clone> Frame<A> {
    fn new(plan: &'_ Plan<A>, bindings: Bindings<'_>, intention_id: IntentionId) -> Self {
        Self {
            intention_id,
            _event: plan.trigger.clone(),
            bindings: bindings.into(),
            remaining: plan.body.iter().rev().cloned().collect(),
        }
    }
}

impl<A> Frame<A> {
    fn step(&mut self, context: &mut Context<A>) -> FrameStepResult {
        let Some(formula) = self.remaining.pop() else {
            return Ok(FrameStep::Done {
                bindings: core::mem::replace(&mut self.bindings, Bindings::empty()),
            });
        };

        let formula = formula.resolve_possible(&self.bindings)?;

        match formula {
            Formula::Belief { trigger, belief } => {
                let event = belief
                    .try_into_ground()
                    .ok_or(FrameStepError::ResolveIncomplete)?
                    // TODO: Avoid the extra conversion here by using an `is_ground`
                    // function.
                    .into_non_ground();

                context.emit_event(
                    TriggeringEvent {
                        trigger,
                        event,
                        goal: None,
                    },
                    self.intention_id,
                )
            }
            Formula::Goal { kind, goal } => context.emit_event(
                TriggeringEvent {
                    trigger: Trigger::Addition,
                    event: goal,
                    goal: Some(kind),
                },
                self.intention_id,
            ),
            Formula::Action(action) => match action {
                Action::System(action) => action.execute(context),
                Action::User(action) => context.perform_action(action),
            },
        }

        // TODO: Immediately return the bindings here if no formula is left.
        Ok(FrameStep::NotDone)
    }
}

type FrameStepResult = core::result::Result<FrameStep, FrameStepError>;

#[derive(Debug)]
enum FrameStep {
    NotDone,
    Done { bindings: OwnedBindings },
}

#[derive(Debug)]
enum FrameStepError {
    ResolveFailure(ResolveFailure),
    ResolveIncomplete,
}

impl core::fmt::Display for FrameStepError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use FrameStepError::*;
        write!(
            f,
            "frame step error: {}",
            match self {
                ResolveFailure(failure) => failure.to_string(),
                ResolveIncomplete => "resolve incomplete".to_string(),
            }
        )
    }
}

impl From<ResolveFailure> for FrameStepError {
    fn from(error: ResolveFailure) -> Self {
        Self::ResolveFailure(error)
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use crate::bindings::Bindings;
    use crate::context::Context;
    use crate::plan::{Action, Formula, Trigger};

    use crate::testing::*;

    use super::*;

    #[test]
    fn test_intention_step_empty() {
        let mut intention: Intention<()> = Intention::default();
        let mut context = Context::new();

        // Step with no frames returns Done
        assert!(matches!(
            intention.step(&mut context),
            IntentionRunResult::Done
        ));
    }

    #[test]
    fn test_intention_push_and_step() {
        let mut intention: Intention<()> = Intention::default();
        let mut context = Context::new();

        let trigger = trigger("event", vec![], None);
        let plan = plan(trigger.clone(), None, vec![]);

        intention.push(&plan, Bindings::empty());

        assert_eq!(intention.stack.len(), 1);

        // Plan has no body, so one step should complete the frame, merge bindings, and remove the frame.
        // It returns Done because the intention has no more frames.
        let result = intention.step(&mut context);
        assert!(matches!(result, IntentionRunResult::Done));
        assert_eq!(intention.stack.len(), 0);
    }

    #[test]
    fn test_intention_step_with_actions() {
        let mut intention: Intention<&'static str> = Intention::default();
        let mut context = Context::new();

        let trigger = trigger("event", vec![], None);
        let plan = plan(
            trigger.clone(),
            None,
            vec![
                Formula::Action(Action::User("action1")),
                Formula::Action(Action::User("action2")),
            ],
        );

        intention.push(&plan, Bindings::empty());

        // step 1: executes action1 (because it's popped first)
        let result = intention.step(&mut context);
        assert!(matches!(result, IntentionRunResult::NotDone));
        assert_eq!(context.actions, &["action1"]);

        // step 2: executes action2
        let result = intention.step(&mut context);
        assert!(matches!(result, IntentionRunResult::NotDone));
        assert_eq!(context.actions, &["action1", "action2"]);

        // step 3: frame done, intention done
        let result = intention.step(&mut context);
        assert!(matches!(result, IntentionRunResult::Done));
    }

    #[test]
    fn test_intention_step_with_beliefs_and_goals() {
        let mut intention: Intention<()> = Intention::default();
        let mut context = Context::new();

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
                },
            ],
        );

        intention.push(&plan, Bindings::empty());

        let result = intention.step(&mut context);
        assert!(matches!(result, IntentionRunResult::NotDone));

        let result = intention.step(&mut context);
        assert!(matches!(result, IntentionRunResult::NotDone));
    }
}
