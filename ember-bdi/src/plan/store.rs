use alloc::collections::{BTreeMap, btree_set::BTreeSet};

use crate::{literal::Literal, term::Atom};

use super::{GoalKind, Plan, Trigger, TriggeringEvent};

#[derive(Debug)]
pub struct PlanStore<A> {
    pub(super) plans: BTreeMap<PlanKey, BTreeSet<Plan<A>>>,
}

impl<A> Default for PlanStore<A> {
    fn default() -> Self {
        Self {
            plans: BTreeMap::default(),
        }
    }
}

impl<A> PlanStore<A> {
    pub fn insert(&mut self, plan: Plan<A>) -> bool {
        self.plans
            .entry((&plan.trigger).into())
            .or_default()
            .insert(plan)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct PlanKey {
    trigger: Trigger,
    event: (Atom, usize),
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
