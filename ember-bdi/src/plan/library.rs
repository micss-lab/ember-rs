use alloc::rc::Rc;
use alloc::vec::Vec;
use core::ops::Deref;

use ember_collections::SmallMap;

use crate::bindings::Bindings;
use crate::context::PureContext;
use crate::knowledge::base::KnowledgeBase;
use crate::term::Atom;

use super::selection::PlanSelection;
use super::selector::PlanSelector;
use super::{GoalKind, Plan, Trigger, TriggeringEvent};

#[derive(Debug)]
pub struct PlanLibrary<A> {
    pub(super) plans: SmallMap<PlanKey, Vec<Plan<A>>>,
}

impl<A> Default for PlanLibrary<A> {
    fn default() -> Self {
        Self {
            plans: SmallMap::default(),
        }
    }
}

impl<A: Ord> PlanLibrary<A> {
    pub fn add(&mut self, plan: Plan<A>) {
        self.plans
            .entry((&plan.trigger).into())
            .or_default()
            .push(plan)
    }
}

impl<A> PlanLibrary<A> {
    pub fn select<'p, 'b, 'e>(
        &'p self,
        event: &'e TriggeringEvent,
        selector: &mut dyn PlanSelector<A>,
        knowledge: &'b KnowledgeBase,
        pure_context: &'b PureContext,
    ) -> Option<(&'p Plan<A>, Bindings<'b>)>
    where
        'p: 'b,
        'e: 'b,
    {
        let selection = PlanSelection::select_from_library(event, &self.plans);
        selector.select_plan(selection, knowledge, pure_context)
    }
}

/// Cheaply-cloneable `Rc` handle to a `PlanLibrary`, shared across every
/// `BdiAgent` built from the same `#[bdi_agent]`-generated type.
#[derive(Debug)]
pub struct SharedPlanLibrary<A>(Rc<PlanLibrary<A>>);

impl<A> Clone for SharedPlanLibrary<A> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<A> SharedPlanLibrary<A> {
    pub fn new(library: PlanLibrary<A>) -> Self {
        Self(Rc::new(library))
    }
}

impl<A> Deref for SharedPlanLibrary<A> {
    type Target = PlanLibrary<A>;

    fn deref(&self) -> &PlanLibrary<A> {
        &self.0
    }
}

impl<A> From<SharedPlanLibrary<A>> for Rc<PlanLibrary<A>> {
    fn from(shared: SharedPlanLibrary<A>) -> Self {
        shared.0
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct PlanKey {
    /// Whether the event is an addition or deletion.
    trigger: Trigger,
    /// What is the class of event that happened.
    event: (Atom, usize),
    /// What should the goal of the plan be.
    goal: Option<GoalKind>,
}

impl From<&TriggeringEvent> for PlanKey {
    fn from(
        TriggeringEvent {
            trigger,
            event,
            goal,
        }: &TriggeringEvent,
    ) -> Self {
        let event = event.atom_and_arity();
        Self {
            trigger: *trigger,
            event,
            goal: *goal,
        }
    }
}
