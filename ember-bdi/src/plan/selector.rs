use crate::bindings::Bindings;
use crate::knowledge::base::BeliefBase;

use super::Plan;
use super::selection::PlanSelection;

/// Select the final plan from the iterator of applicable plans.
///
/// NOTE: Currently this interface only supports selecting a plan based on the single plan itself.
/// Future versions might allow a context of multiple plans to choose the best one.
pub trait PlanSelector {
    fn select_plan<'p, 'e, 'b, A>(
        &mut self,
        mut selection: PlanSelection<'p, 'e, A>,
        knowledge: &'b BeliefBase,
    ) -> Option<(&'p Plan<A>, Bindings<'b>)>
    where
        'p: 'b,
        'e: 'b,
    {
        while let Some((plan, bindings)) = selection.next_plan(knowledge) {
            if let Some(plan) = self.filter_plan(plan) {
                return Some((plan, bindings));
            }
        }
        None
    }

    fn filter_plan<'p, A>(&mut self, plan: &'p Plan<A>) -> Option<&'p Plan<A>>;
}

/// Selects the first applicable plan from the plan library.
pub struct FirstApplicable;

impl PlanSelector for FirstApplicable {
    fn filter_plan<'p, A>(&mut self, plan: &'p Plan<A>) -> Option<&'p Plan<A>> {
        Some(plan)
    }
}
