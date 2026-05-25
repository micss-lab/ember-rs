use alloc::collections::{BTreeMap, BTreeSet};

use crate::bindings::Bindings;
use crate::knowledge::store::BeliefBase;
use crate::literal::Literal;
use crate::term::Atom;

use super::selection::PlanSelection;
use super::selector::PlanSelector;
use super::{GoalKind, Plan, Trigger, TriggeringEvent};

#[derive(Debug)]
pub struct PlanLibrary<A> {
    pub(super) plans: BTreeMap<PlanKey, BTreeSet<Plan<A>>>,
}

impl<A> Default for PlanLibrary<A> {
    fn default() -> Self {
        Self {
            plans: BTreeMap::default(),
        }
    }
}

impl<A> PlanLibrary<A> {
    pub fn add(&mut self, plan: Plan<A>) -> bool {
        self.plans
            .entry((&plan.trigger).into())
            .or_default()
            .insert(plan)
    }

    pub fn select<'p, 'b, 'e, S>(
        &'p mut self,
        event: &'e TriggeringEvent,
        knowledge: &'b BeliefBase,
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
        let event = match event {
            Literal::Atom { structure, .. } => structure.atom_and_arity(),
            Literal::Variable(_) => unimplemented!(
                "single variable as triggering event of a plan is not supported (yet)"
            ),
        };
        Self {
            trigger: *trigger,
            event,
            goal: *goal,
        }
    }
}
