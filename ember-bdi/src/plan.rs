use alloc::rc::Rc;

use derive_where::derive_where;

use crate::literal::{Literal, LiteralView};

pub use crate::event::{GoalKind, Trigger, TriggeringEvent};
pub use crate::knowledge::query::formula::*;

pub use self::action::{Action, BuiltinAction, ImpureAction, PureAction};

pub mod action;
pub mod library;
pub mod selection;
pub mod selector;

#[derive(Debug)]
pub struct Plan<A> {
    pub trigger: TriggeringEvent,
    pub context: Option<QueryFormula>,
    /// # Why Rc?
    ///
    /// A plan body is cloned often to be stored in an intention's frame as a list of remaining
    /// steps. Storing a pure reference is not possible as the lifetime would be
    /// self-refferential inside a bdi agent. Hence an Rc is this best option.
    pub body: Rc<[Formula<A>]>,
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

#[derive_where(Debug, PartialEq, Eq)]
#[derive(Clone)]
pub enum Formula<A> {
    Belief {
        trigger: Trigger,
        belief: Literal,
        /// Should updating the beliefbase emit an event for this one.
        silent: bool,
    },
    Goal {
        kind: GoalKind,
        goal: Literal,
    },
    Unify {
        lhs: ArithmeticExpression,
        rhs: ArithmeticExpression,
    },
    Action(Action<A>),
}

pub enum FormulaView<'a, A> {
    Formula(&'a Formula<A>),
    Belief {
        trigger: Trigger,
        belief: LiteralView<'a>,
        silent: bool,
    },
    Goal {
        kind: GoalKind,
        goal: LiteralView<'a>,
    },
}

impl<A> FormulaView<'_, A> {
    pub(crate) fn to_owned(&self) -> Formula<A> {
        todo!()
    }
}
