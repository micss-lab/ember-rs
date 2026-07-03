use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::bindings::Bindings;
use crate::knowledge::base::KnowledgeBase;
use crate::term::Atom;

use super::selection::PlanSelection;
use super::selector::PlanSelector;
use super::{GoalKind, Plan, Trigger, TriggeringEvent};

#[derive(Debug)]
pub struct PlanLibrary<A> {
    pub(super) plans: BTreeMap<PlanKey, Vec<Plan<A>>>,
}

impl<A> Default for PlanLibrary<A> {
    fn default() -> Self {
        Self {
            plans: BTreeMap::default(),
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
    pub fn select<'p, 'b, 'e, S>(
        &'p mut self,
        event: &'e TriggeringEvent,
        knowledge: &'b KnowledgeBase,
        mut selector: S,
    ) -> Option<(&'p Plan<A>, Bindings<'b>)>
    where
        'p: 'b,
        'e: 'b,
        S: PlanSelector,
    {
        let selection = PlanSelection::select_from_library(event, self);
        selector.select_plan(selection, knowledge)
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
