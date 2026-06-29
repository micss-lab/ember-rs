use core::slice::Iter;

use crate::bindings::Bindings;
use crate::knowledge::base::BeliefBase;
use crate::term::view::TermView;

use super::library::PlanLibrary;
use super::{Plan, TriggeringEvent};

/// Lazy selector of plans that are relevant and applicable.
#[derive(Debug)]
pub struct PlanSelection<'p, 'e, A> {
    /// Iterator of plans that might be relevant. They match closely in triggering event, but
    /// require unification with the event before being sure.
    plans: Option<Iter<'p, Plan<A>>>,

    event: &'e TriggeringEvent,
}

impl<'p, 'e, A> PlanSelection<'p, 'e, A> {
    pub fn select_from_library(event: &'e TriggeringEvent, library: &'p PlanLibrary<A>) -> Self {
        Self {
            plans: library.plans.get(&event.into()).map(|p| p.iter()),
            event,
        }
    }

    pub fn next_plan<'b>(
        &mut self,
        knowledge: &'b BeliefBase,
    ) -> Option<(&'p Plan<A>, Bindings<'b>)>
    where
        'e: 'b,
        'p: 'b,
    {
        self.relevant().applicable().next_plan(knowledge)
    }

    fn relevant<'s>(&'s mut self) -> RelevantPlanSelection<'s, 'p, 'e, A> {
        RelevantPlanSelection {
            plans: self.plans.as_mut(),
            event: self.event,
        }
    }
}

#[derive(Debug)]
struct RelevantPlanSelection<'s, 'p, 'e, A> {
    plans: Option<&'s mut Iter<'p, Plan<A>>>,

    event: &'e TriggeringEvent,
}

impl<'s, 'p, 'e, A> RelevantPlanSelection<'s, 'p, 'e, A> {
    fn applicable(self) -> ApplicablePlanSelection<'s, 'p, 'e, A> {
        ApplicablePlanSelection(self)
    }
}

impl<'p, 'e, A> RelevantPlanSelection<'_, 'p, 'e, A> {
    fn next_plan<'b>(&mut self) -> Option<(&'p Plan<A>, Bindings<'e>)>
    where
        'p: 'e,
    {
        use crate::unification::traits::UnifyView;

        // NOTE: It is assumed that the iterator of plans all have the correct trigger and
        // goal.
        self.plans.as_mut()?.find_map(|p| {
            TermView::from(&self.event.event)
                .unify(TermView::from(&p.trigger.event), None)
                .ok()
                .map(|b| (p, b))
        })
    }
}

#[derive(Debug)]
struct ApplicablePlanSelection<'s, 'p, 'e, A>(RelevantPlanSelection<'s, 'p, 'e, A>);

impl<'p, 'e, A> ApplicablePlanSelection<'_, 'p, 'e, A> {
    fn next_plan<'b>(&mut self, knowledge: &'b BeliefBase) -> Option<(&'p Plan<A>, Bindings<'b>)>
    where
        'p: 'e,
        'e: 'b,
    {
        use crate::knowledge::query::IntoQuery;

        while let Some((relevant_plan, bindings)) = self.0.next_plan() {
            let Some(ref context) = relevant_plan.context else {
                return Some((relevant_plan, bindings));
            };

            let Some(bindings) = context.into_query(knowledge).next_bindings(Some(&bindings))
            else {
                continue;
            };

            return Some((relevant_plan, bindings));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;
    use alloc::vec;

    use crate::knowledge::base::BeliefBase;
    use crate::literal::Literal;
    use crate::plan::{GoalKind, QueryFormula};
    use crate::term::{Atom, Term};
    use crate::testing::*;

    use super::*;

    #[test]
    fn test_relevant_but_not_applicable() {
        let mut store = PlanLibrary::<()>::default();
        let mut bb = BeliefBase::default();

        // Plan: +!test : is_ready <- ...
        let plan = Plan {
            trigger: trigger("test", vec![], Some(GoalKind::Achieve)),
            context: Some(crate::plan::QueryFormula::Literal(Literal::Atom {
                negated: false,
                structure: crate::term::Structure {
                    functor: Atom("is_ready".into()),
                    arguments: None,
                },
            })),
            body: Box::new([]),
        };
        store.add(plan);

        let event = trigger("test", vec![], Some(GoalKind::Achieve));
        let mut selection = PlanSelection::select_from_library(&event, &store);

        // 1. If belief base is empty, context fails.
        assert!(
            selection.next_plan(&bb).is_none(),
            "Plan should not be applicable"
        );

        // 2. Add the belief, now it should be applicable.
        let ready_belief = Literal::Atom {
            negated: false,
            structure: crate::term::Structure {
                functor: Atom("is_ready".into()),
                arguments: None,
            },
        };
        bb.assert_no_event(ready_belief);

        let mut selection2 = PlanSelection::select_from_library(&event, &store);
        assert!(
            selection2.next_plan(&bb).is_some(),
            "Plan should now be applicable"
        );
    }

    #[test]
    fn test_backtracking_to_second_plan() {
        let mut store = PlanLibrary::<()>::default();
        let bb = BeliefBase::default();

        // Plan 1: +!goal : false_context <- ... (Should fail context)
        store.add(Plan {
            trigger: trigger("goal", vec![], Some(GoalKind::Achieve)),
            context: Some(crate::plan::QueryFormula::Literal(Literal::Atom {
                negated: false,
                structure: crate::term::Structure {
                    functor: Atom("never".into()),
                    arguments: None,
                },
            })),
            body: Box::new([]),
        });

        // Plan 2: +!goal : true (Should succeed)
        store.add(Plan {
            trigger: trigger("goal", vec![], Some(GoalKind::Achieve)),
            context: None,
            body: Box::new([]),
        });

        let event = trigger("goal", vec![], Some(GoalKind::Achieve));
        let mut selection = PlanSelection::select_from_library(&event, &store);

        let result = selection.next_plan(&bb);
        assert!(result.is_some(), "Should skip Plan 1 and find Plan 2");
        assert!(
            result.unwrap().0.context.is_none(),
            "Should have selected the second plan"
        );
    }

    #[test]
    fn test_unification_failure_in_relevance() {
        let mut store = PlanLibrary::<()>::default();
        let bb = BeliefBase::default();

        // Plan for test(1)
        store.add(Plan {
            trigger: trigger("test", vec![number(1.0)], Some(GoalKind::Achieve)),
            context: None,
            body: Box::new([]),
        });

        // Event for test(2)
        let event = trigger("test", vec![number(2.0)], Some(GoalKind::Achieve));
        let mut selection = PlanSelection::select_from_library(&event, &store);

        assert!(
            selection.next_plan(&bb).is_none(),
            "Plan trigger test(1) should not match event test(2)"
        );
    }

    #[test]
    fn test_variable_unification_event_to_plan() {
        let mut store = PlanLibrary::<()>::default();
        let bb = BeliefBase::default();

        let x = variable();
        // Plan: +!greet(Name)
        store.add(Plan {
            trigger: trigger("greet", vec![variable_term(&x)], Some(GoalKind::Achieve)),
            context: None,
            body: Box::new([]),
        });

        // Event: !greet("Alice")
        let event = trigger("greet", vec![string("Alice")], Some(GoalKind::Achieve));
        let mut selection = PlanSelection::select_from_library(&event, &store);

        let (_, bindings) = selection.next_plan(&bb).expect("Should unify");

        // Check that X was correctly bound to "Alice"
        assert_eq!(bindings.get_view(&x), Some(&string("Alice").as_view()));
    }

    #[test]
    fn test_empty_store_returns_none() {
        let store = PlanLibrary::<()>::default();
        let bb = BeliefBase::default();
        let event = trigger("any", vec![], None);

        let mut selection = PlanSelection::select_from_library(&event, &store);
        assert!(selection.next_plan(&bb).is_none());
    }

    #[test]
    fn test_context_uses_trigger_bindings() {
        let mut store = PlanLibrary::<()>::default();
        let mut bb = BeliefBase::default();

        // Belief: colour(circle, red)
        let colour_belief = Literal::Atom {
            negated: false,
            structure: crate::term::Structure {
                functor: Atom("colour".into()),
                arguments: Some(Box::new([string("circle"), string("red")])),
            },
        };
        bb.assert_no_event(colour_belief);

        let x = variable();
        // Plan: +!check(Obj) : colour(Obj, red) <- ...
        store.add(Plan {
            trigger: trigger("check", vec![variable_term(&x)], Some(GoalKind::Achieve)),
            context: Some(crate::plan::QueryFormula::Literal(Literal::Atom {
                negated: false,
                structure: crate::term::Structure {
                    functor: Atom("colour".into()),
                    arguments: Some(Box::new([variable_term(&x), string("red")])),
                },
            })),
            body: Box::new([]),
        });

        // Event: !check("circle")
        let event = trigger("check", vec![string("circle")], Some(GoalKind::Achieve));
        let mut selection = PlanSelection::select_from_library(&event, &store);

        let result = selection.next_plan(&bb);
        assert!(
            result.is_some(),
            "Context should see that Obj is 'circle' from the event"
        );
    }

    #[test]
    fn test_full_binding_propagation_pipeline() {
        let mut store = PlanLibrary::<()>::default();
        let mut bb = BeliefBase::default();
        assert_belief(&mut bb, "color", vec![string("apple"), string("red")]);

        let (x, y) = (variable(), variable());
        store.add(Plan {
            trigger: trigger("check", vec![variable_term(&x)], Some(GoalKind::Achieve)),
            context: Some(literal_formula(
                "color",
                vec![variable_term(&x), variable_term(&y)],
            )),
            body: Box::new([]),
        });

        let event = trigger("check", vec![string("apple")], Some(GoalKind::Achieve));
        let mut selection = PlanSelection::select_from_library(&event, &store);
        let (_, bindings) = selection.next_plan(&bb).expect("Binding pipe failed");

        assert_eq!(bindings.get_view(&x), Some(&string("apple").as_view()));
        assert_eq!(bindings.get_view(&y), Some(&string("red").as_view()));
    }

    #[test]
    fn test_variable_aliasing_event_to_context() {
        let mut store = PlanLibrary::<()>::default();
        let mut bb = BeliefBase::default();
        assert_belief(&mut bb, "linked", vec![string("a"), string("b")]);

        let (event_var, plan_var) = (variable(), variable());
        store.add(Plan {
            trigger: trigger(
                "connect",
                vec![variable_term(&plan_var)],
                Some(GoalKind::Achieve),
            ),
            context: Some(literal_formula(
                "linked",
                vec![variable_term(&plan_var), Term::Variable(variable())],
            )),
            body: Box::new([]),
        });

        let event = trigger(
            "connect",
            vec![variable_term(&event_var)],
            Some(GoalKind::Achieve),
        );
        let mut selection = PlanSelection::select_from_library(&event, &store);
        let (_, bindings) = selection.next_plan(&bb).expect("Aliasing failed");

        assert_eq!(bindings.get_view(&event_var), Some(&string("a").as_view()));
    }

    #[test]
    fn test_backtracking_on_context_failure() {
        let mut store = PlanLibrary::<()>::default();
        let mut bb = BeliefBase::default();
        assert_belief(&mut bb, "is_broken", vec![string("bolt")]);

        let x = variable();
        store.add(Plan {
            trigger: trigger("fix", vec![variable_term(&x)], Some(GoalKind::Achieve)),
            context: Some(literal_formula("is_tool", vec![variable_term(&x)])), // Fails
            body: Box::new([]),
        });
        store.add(Plan {
            trigger: trigger("fix", vec![variable_term(&x)], Some(GoalKind::Achieve)),
            context: Some(literal_formula("is_broken", vec![variable_term(&x)])), // Succeeds
            body: Box::new([]),
        });

        let event = trigger("fix", vec![string("bolt")], Some(GoalKind::Achieve));
        let (plan, _) = PlanSelection::select_from_library(&event, &store)
            .next_plan(&bb)
            .unwrap();

        let QueryFormula::Literal(Literal::Atom { structure, .. }) = plan.context.as_ref().unwrap()
        else {
            unreachable!()
        };
        assert_eq!(structure.functor.0, "is_broken");
    }

    #[test]
    fn test_context_negation_with_event_bindings() {
        let mut bb = BeliefBase::default();
        let mut store = PlanLibrary::<()>::default();
        assert_belief(&mut bb, "blocked", vec![string("north")]);

        let dir = variable();
        store.add(Plan {
            trigger: trigger("move", vec![variable_term(&dir)], Some(GoalKind::Achieve)),
            context: Some(QueryFormula::Not(Box::new(literal_formula(
                "blocked",
                vec![variable_term(&dir)],
            )))),
            body: Box::new([]),
        });

        let event_north = trigger("move", vec![string("north")], Some(GoalKind::Achieve));
        assert!(
            PlanSelection::select_from_library(&event_north, &store)
                .next_plan(&bb)
                .is_none()
        );

        let event_south = trigger("move", vec![string("south")], Some(GoalKind::Achieve));
        assert!(
            PlanSelection::select_from_library(&event_south, &store)
                .next_plan(&bb)
                .is_some()
        );
    }
}
