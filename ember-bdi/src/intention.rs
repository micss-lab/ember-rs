use alloc::vec::Vec;

use derive_where::derive_where;

use crate::bindings::Bindings;
use crate::context::Context;
use crate::plan::{Formula, Trigger, TriggeringEvent};

#[derive_where(Default)]
pub(crate) struct Intention<'b, A> {
    stack: Vec<Frame<'b, A>>,
}

impl<'b, A> Intention<'b, A> {
    fn step(&mut self, context: &mut Context<A>) -> IntentionRunResult {
        let Some(frame) = self.stack.last_mut() else {
            return IntentionRunResult::Done;
        };

        let FrameStepResult::Done { bindings } = frame.step(context) else {
            return IntentionRunResult::NotDone;
        };

        self.stack.pop();

        let Some(frame) = self.stack.last_mut() else {
            return IntentionRunResult::Done;
        };

        todo!("merge bindings here");
        IntentionRunResult::NotDone;
    }
}

pub(crate) enum IntentionRunResult {
    NotDone,
    Done,
}

struct Frame<'b, A> {
    /// The event that triggered the creation of this frame. Used to stop the execution of
    /// plans.
    event: TriggeringEvent,
    /// Bindings that this frame is created with and that have been resolved during the
    /// execution of this frame.
    bindings: Bindings<'b>,
    /// Remaining parts of the plan body to execute.
    remaining: Vec<Formula<A>>,
}

impl<'b, A> Frame<'b, A> {
    fn step(&mut self, context: &mut Context<A>) -> FrameStepResult<'b> {
        let Some(formula) = self.remaining.pop() else {
            return FrameStepResult::Done {
                bindings: core::mem::replace(&mut self.bindings, Bindings::empty()),
            };
        };

        match formula {
            Formula::Belief { trigger, belief } => context.emit_event(TriggeringEvent {
                trigger,
                event: todo!("implement resolving of a literal"),
                goal: None,
            }),
            Formula::Goal { kind, goal } => context.emit_event(TriggeringEvent {
                trigger: Trigger::Addition,
                event: todo!("implement resolving of a literal"),
                goal: Some(kind),
            }),
            Formula::Action(action) => context.perform_action(action),
        }

        FrameStepResult::NotDone
    }
}

// TODO: Add `failed` as a possible variant.
enum FrameStepResult<'b> {
    NotDone,
    Done { bindings: Bindings<'b> },
}
