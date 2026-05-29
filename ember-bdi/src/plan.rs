use alloc::boxed::Box;

use crate::literal::Literal;

pub use crate::event::{GoalKind, Trigger, TriggeringEvent};
pub use crate::knowledge::query::formula::*;

pub use self::action::{Action, SystemAction};
use crate::bindings::BindingLookup;
use crate::bindings::resolver::ResolveFailure;

pub mod action;
pub mod library;
pub mod selection;
pub mod selector;

#[derive(Debug)]
pub struct Plan<A> {
    pub trigger: TriggeringEvent,
    pub context: Option<QueryFormula>,
    pub body: Box<[Formula<A>]>,
}

impl<A> PartialEq for Plan<A> {
    fn eq(&self, other: &Self) -> bool {
        PlanEqOrd::from(self) == PlanEqOrd::from(other)
    }
}

impl<A> Eq for Plan<A> {}

impl<A> PartialOrd for Plan<A> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<A> Ord for Plan<A> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        PlanEqOrd::from(self).cmp(&PlanEqOrd::from(other))
    }
}

#[derive(PartialEq, Eq)]
struct PlanEqOrd<'a> {
    trigger: &'a TriggeringEvent,
    context: &'a Option<QueryFormula>,
}

impl<'a> PartialOrd for PlanEqOrd<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for PlanEqOrd<'a> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.trigger
            .cmp(other.trigger)
            .then_with(|| match (self.context, other.context) {
                (Some(_), None) => core::cmp::Ordering::Less,
                (None, Some(_)) => core::cmp::Ordering::Greater,
                _ => self.context.cmp(other.context),
            })
    }
}

impl<'a, A> From<&'a Plan<A>> for PlanEqOrd<'a> {
    fn from(
        Plan {
            trigger, context, ..
        }: &'a Plan<A>,
    ) -> Self {
        Self { trigger, context }
    }
}

#[derive(Debug, Clone)]
pub enum Formula<A> {
    Belief { trigger: Trigger, belief: Literal },
    Goal { kind: GoalKind, goal: Literal },
    Action(Action<A>),
}

impl<A> Formula<A> {
    pub(crate) fn resolve_possible<B: BindingLookup>(
        self,
        bindings: &B,
    ) -> Result<Self, ResolveFailure> {
        Ok(match self {
            Formula::Belief { trigger, belief } => Formula::Belief {
                trigger,
                belief: belief.resolve_possible(bindings)?,
            },
            Formula::Goal { kind, goal } => Formula::Goal {
                kind,
                goal: goal.resolve_possible(bindings)?,
            },
            action @ Formula::Action(_) => action,
        })
    }
}
