use alloc::string::ToString;
use alloc::vec::Vec;

use derive_where::derive_where;

use crate::bindings::Bindings;
use crate::bindings::resolver::ResolveFailure;
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

        let bindings = match frame.step(context) {
            Ok(FrameStep::Done { bindings }) => bindings,
            Ok(FrameStep::NotDone) => return IntentionRunResult::NotDone,
            Err(_) => unimplemented!("handle the error"),
        };

        self.stack.pop();

        let Some(frame) = self.stack.last_mut() else {
            return IntentionRunResult::Done;
        };

        frame.bindings = Bindings::merge([&frame.bindings, &bindings])
            .expect("merging bindings between frames failed");
        IntentionRunResult::NotDone
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
            return Ok(FrameStep::Done {
                bindings: core::mem::replace(&mut self.bindings, Bindings::empty()),
            });
        };

        match formula {
            Formula::Belief { trigger, belief } => {
                let event = belief
                    .resolve_possible(&self.bindings)?
                    .try_into_ground()
                    .ok_or(FrameStepError::ResolveIncomplete)?
                    // TODO: Avoid the extra conversion here by using an `is_ground`
                    // function.
                    .into_non_ground();

                context.emit_event(TriggeringEvent {
                    trigger,
                    event,
                    goal: None,
                })
            }
            Formula::Goal { kind, goal } => {
                let event = goal.resolve_possible(&self.bindings)?;
                context.emit_event(TriggeringEvent {
                    trigger: Trigger::Addition,
                    event,
                    goal: Some(kind),
                })
            }
            Formula::Action(action) => context.perform_action(action),
        }

        // TODO: Immediately return the bindings here if no formula is left.
        Ok(FrameStep::NotDone)
    }
}

type FrameStepResult<'b> = core::result::Result<FrameStep<'b>, FrameStepError>;

#[derive(Debug)]
enum FrameStep<'b> {
    NotDone,
    Done { bindings: Bindings<'b> },
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
