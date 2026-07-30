use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::bindings::Bindings;
use crate::knowledge::base::KnowledgeBase;
use crate::term::Atom;

use super::selection::PlanSelection;
use super::selector::{FirstApplicable, PlanSelector};
use super::{GoalKind, Plan, Trigger, TriggeringEvent};

#[derive(Debug)]
pub struct PlanLibrary<A, PSel = FirstApplicable> {
    pub(super) plans: BTreeMap<PlanKey, Vec<Plan<A>>>,
    selector: PSel,
}

impl<A, PSel: Default> Default for PlanLibrary<A, PSel> {
    fn default() -> Self {
        Self {
            plans: BTreeMap::default(),
            selector: PSel::default(),
        }
    }
}

impl<A: Ord, PSel> PlanLibrary<A, PSel> {
    pub fn add(&mut self, plan: Plan<A>) {
        self.plans
            .entry((&plan.trigger).into())
            .or_default()
            .push(plan)
    }
}

impl<A, PSel> PlanLibrary<A, PSel> {
    /// Configures how a plan is chosen among those applicable to an event. Replaces the default
    /// (`FirstApplicable`). Changes the selector's type, so it returns a differently-typed
    /// library.
    pub fn with_plan_selector<NewPSel>(self, selector: NewPSel) -> PlanLibrary<A, NewPSel> {
        PlanLibrary {
            plans: self.plans,
            selector,
        }
    }

    pub fn select<'p, 'b, 'e>(
        &'p mut self,
        event: &'e TriggeringEvent,
        knowledge: &'b KnowledgeBase,
    ) -> Option<(&'p Plan<A>, Bindings<'b>)>
    where
        'p: 'b,
        'e: 'b,
        PSel: PlanSelector<A>,
    {
        let selection = PlanSelection::select_from_library(event, &self.plans);
        self.selector.select_plan(selection, knowledge)
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
