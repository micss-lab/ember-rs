use alloc::vec::Vec;

use derive_where::derive_where;

use crate::bindings::{Bindings, OwnedBindings};
use crate::context::Context;
use crate::knowledge::base::KnowledgeBase;
use crate::plan::action::{Execute, ExecuteResult, PendingAction};
use crate::plan::{Formula, Plan, Trigger, TriggeringEvent};

use self::result::*;

pub mod queue;
pub(crate) mod result;

pub use self::queue::{Fifo, Scheduler};

pub type IntentionId = usize;

#[derive_where(Debug)]
pub struct Intention<A> {
    id: IntentionId,
    stack: Vec<Frame<A>>,
}

impl<A, S> Intention<A>
where
    A: Execute<State = S, UserAction = A>,
{
    pub(crate) fn step(
        &mut self,
        context: &mut Context<A>,
        knowledge: &mut KnowledgeBase,
        state: &mut S,
    ) -> Result {
        let Some(frame) = self.stack.last_mut() else {
            return StepOk::done();
        };

        let bindings = match frame.step(context, knowledge, state)? {
            StepOk::Done => frame.take_filtered_bindings(),
            StepOk::Pending => return StepOk::pending(),
        };

        self.stack.pop(); // Remove the done frame

        let Some(next_frame) = self.stack.last_mut() else {
            return StepOk::done();
        };

        next_frame.bindings = Bindings::merge([
            bindings,
            core::mem::replace(&mut next_frame.bindings, Bindings::empty()),
        ])
        .expect("merging bindings between frames failed");

        StepOk::pending()
    }
}

impl<A> Intention<A> {
    fn new(id: IntentionId) -> Self {
        Self {
            id,
            stack: Vec::default(),
        }
    }

    #[cfg(test)]
    pub(crate) fn stack_len(&self) -> usize {
        self.stack.len()
    }
}

impl<A: Clone> Intention<A> {
    pub(crate) fn push(
        &mut self,
        plan: &'_ Plan<A>,
        bindings: Bindings<'_>,
        event: TriggeringEvent,
    ) {
        self.stack.push(Frame::new(plan, bindings, self.id, event))
    }
}

#[derive_where(Debug)]
struct Frame<A> {
    /// The id of the intention this frame belongs to.
    intention_id: IntentionId,
    /// The event that triggered the creation of this frame. Used to stop the execution of
    /// plans, and to filter bindings.
    event: TriggeringEvent,
    /// Bindings that this frame is created with and that have been resolved during the
    /// execution of this frame.
    bindings: OwnedBindings,
    /// Remaining parts of the plan body to execute.
    remaining: Vec<Formula<A>>,
}

impl<A: Clone> Frame<A> {
    fn new(
        plan: &'_ Plan<A>,
        bindings: Bindings<'_>,
        intention_id: IntentionId,
        event: TriggeringEvent,
    ) -> Self {
        Self {
            intention_id,
            event,
            bindings: bindings.into(),
            remaining: plan.body.iter().rev().cloned().collect(),
        }
    }
}

impl<A, S> Frame<A>
where
    A: Execute<State = S, UserAction = A>,
{
    fn step(
        &mut self,
        context: &mut Context<A>,
        knowledge: &mut KnowledgeBase,
        state: &mut S,
    ) -> Result {
        let Some(formula) = self.remaining.pop() else {
            return StepOk::done();
        };

        let formula = formula.resolve_possible(&self.bindings)?;

        // Prevent tail recursion optimization if the last frame dispatched an action to the agent.
        let mut blocked_on_pending_action = false;

        match formula {
            Formula::Belief {
                trigger,
                belief,
                silent,
            } => {
                let event = if !belief.is_ground() {
                    return Err(StepError::ResolveIncomplete);
                } else {
                    belief
                };
                match trigger {
                    Trigger::Addition => {
                        if silent {
                            knowledge.assert_no_event(event);
                        } else {
                            knowledge.assert(event, context, Some(self.intention_id));
                        }
                    }
                    Trigger::Deletion => {
                        if silent {
                            knowledge.remove_no_event(event);
                        } else {
                            knowledge.remove(event, context, Some(self.intention_id));
                        }
                    }
                }
            }
            Formula::Goal { kind, goal } => context.emit_event(
                TriggeringEvent {
                    trigger: Trigger::Addition,
                    event: goal,
                    goal: Some(kind),
                },
                Some(self.intention_id),
            ),
            Formula::Unify { lhs, rhs } => {
                use crate::knowledge::query::formula::eval::evaluate_relational;
                use crate::plan::{RelationalOperator, RelationalQueryFormula};

                let formula = RelationalQueryFormula {
                    operator: RelationalOperator::Unify,
                    operands: (lhs, rhs),
                };
                match evaluate_relational(&formula, &self.bindings.as_bindings()) {
                    Ok(Some(bindings)) => {
                        self.bindings =
                            Bindings::merge_views([&self.bindings.as_bindings(), &bindings])
                                .expect("merging bindings from unify into frame failed")
                                .into()
                    }
                    Ok(None) => return Err(StepError::UnifyFailed),
                    Err(error) => return Err(StepError::UnifyEvalError(error)),
                }
            }
            Formula::Action(action) => {
                use crate::plan::action::Execute;

                // TODO: Make use of resolve to resolve actions as well.
                match action.execute(&self.bindings, &mut *context, knowledge, state) {
                    ExecuteResult::Pending(pending) => {
                        let intention = pending
                            .should_block_intention()
                            .then_some(self.intention_id);
                        blocked_on_pending_action = intention.is_some();
                        context.dispatch_action(
                            PendingAction::new(pending, self.bindings.clone()),
                            intention,
                        )
                    }
                    ExecuteResult::Done(Some(bindings)) => {
                        self.bindings =
                            Bindings::merge_views([&self.bindings.as_bindings(), &bindings])
                                .expect("merging bindings from action into frame failed")
                                .into()
                    }
                    ExecuteResult::Done(None) => (),
                }
            }
        }

        // Pop the frame immediately such that it does not leak when the plan is infinitely tail
        // recursive. For example, heartbeat plans.
        if self.remaining.is_empty() && !blocked_on_pending_action {
            return StepOk::done();
        }

        StepOk::pending()
    }

    fn take_filtered_bindings(&mut self) -> OwnedBindings {
        let vars = self.event.event.variables();
        let mut map = alloc::collections::BTreeMap::new();

        if let Some(bindings) = &self.bindings.bindings {
            for v_id in vars {
                if let Some(val) = bindings.get(&v_id) {
                    map.insert(v_id, val.clone());
                }
            }
        }

        OwnedBindings::new(map, crate::bindings::AliasMap::empty())
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;
    use alloc::vec;

    use crate::bindings::Bindings;
    use crate::plan::{Action, ArithmeticExpression, ArithmeticOperator, Formula, Trigger};
    use crate::variable::Variable;

    use crate::testing::*;

    use super::*;

    #[test]
    fn test_intention_step_empty() {
        let mut intention: Intention<()> = Intention::new(0);
        // SAFETY: The environment on the context remains untouched,
        let mut context = unsafe { new_context_without_environment() };
        let mut knowledge = KnowledgeBase::default();

        // Step with no frames returns Done
        assert!(matches!(
            intention.step(&mut context, &mut knowledge, &mut ()),
            Ok(StepOk::Done)
        ));
    }

    #[test]
    fn test_intention_push_and_step() {
        let mut intention: Intention<()> = Intention::new(0);
        // SAFETY: The environment on the context remains untouched,
        let mut context = unsafe { new_context_without_environment() };
        let mut knowledge = KnowledgeBase::default();

        let trigger = trigger("event", vec![], None);
        let plan = plan(trigger.clone(), None, vec![]);

        intention.push(&plan, Bindings::empty(), trigger);

        assert_eq!(intention.stack.len(), 1);

        // Plan has no body, so one step should complete the frame, merge bindings, and remove the frame.
        // It returns Done because the intention has no more frames.
        let result = intention.step(&mut context, &mut knowledge, &mut ());
        assert!(matches!(result, Ok(StepOk::Done)));
        assert_eq!(intention.stack.len(), 0);
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct LogAction(&'static str);

    impl crate::plan::action::Execute for LogAction {
        type State = Vec<&'static str>;
        type UserAction = LogAction;

        fn execute<'b, B>(
            self,
            _bindings: &B,
            _context: &mut Context<Self::UserAction>,
            _knowledge: &KnowledgeBase,
            state: &mut Self::State,
        ) -> crate::plan::action::ExecuteResult<'b, Self>
        where
            B: crate::bindings::BindingLookup + 'b,
        {
            state.push(self.0);
            crate::plan::action::ExecuteResult::Done(None)
        }
    }

    #[test]
    fn test_intention_step_with_actions() {
        let mut intention: Intention<LogAction> = Intention::new(0);
        // SAFETY: The environment on the context remains untouched,
        let mut context = unsafe { new_context_without_environment() };
        let mut knowledge = KnowledgeBase::default();
        let mut state = Vec::new();

        let trigger = trigger("event", vec![], None);
        let plan = plan(
            trigger.clone(),
            None,
            vec![
                Formula::Action(Action::User(LogAction("action1"))),
                Formula::Action(Action::User(LogAction("action2"))),
            ],
        );

        intention.push(&plan, Bindings::empty(), trigger);

        // step 1: executes action1 (because it's popped first), one more formula left in the
        // body so the frame isn't done yet.
        let result = intention.step(&mut context, &mut knowledge, &mut state);
        assert!(matches!(result, Ok(StepOk::Pending)));
        assert_eq!(state, vec!["action1"]);

        // step 2: executes action2, the last formula in the body - both it and the (empty)
        // body being exhausted happen in this same step, so the frame reports done immediately
        // rather than needing an extra no-op step to notice.
        let result = intention.step(&mut context, &mut knowledge, &mut state);
        assert!(matches!(result, Ok(StepOk::Done)));
        assert_eq!(state, vec!["action1", "action2"]);
    }

    #[test]
    fn test_intention_step_with_beliefs_and_goals() {
        let mut intention: Intention<()> = Intention::new(0);
        // SAFETY: The environment on the context remains untouched,
        let mut context = unsafe { new_context_without_environment() };
        let mut knowledge = KnowledgeBase::default();

        let trigger = trigger("event", vec![], None);
        let plan = plan(
            trigger.clone(),
            None,
            vec![
                Formula::Goal {
                    kind: crate::plan::GoalKind::Achieve,
                    goal: literal("goal1", Vec::with_capacity(0)),
                },
                Formula::Belief {
                    trigger: Trigger::Addition,
                    belief: literal("belief1", Vec::with_capacity(0)),
                    silent: false,
                },
            ],
        );

        intention.push(&plan, Bindings::empty(), trigger);

        // step 1: emits the goal event, one more formula (the belief) left in the body.
        let result = intention.step(&mut context, &mut knowledge, &mut ());
        assert!(matches!(result, Ok(StepOk::Pending)));

        // step 2: asserts the belief, the last formula in the body - done immediately, same as
        // the all-actions case above.
        let result = intention.step(&mut context, &mut knowledge, &mut ());
        assert!(matches!(result, Ok(StepOk::Done)));
    }

    #[test]
    fn belief_formula_is_queryable_immediately_after_its_own_step() {
        use crate::knowledge::query::IntoQuery;

        let mut intention: Intention<()> = Intention::new(0);
        // SAFETY: The environment on the context remains untouched.
        let mut context = unsafe { new_context_without_environment() };
        let mut knowledge = KnowledgeBase::default();

        let trigger = trigger("event", vec![], None);
        let plan = plan(
            trigger.clone(),
            None,
            vec![Formula::Belief {
                trigger: Trigger::Addition,
                belief: literal("route_active", vec![string("load"), string("c")]),
                silent: false,
            }],
        );

        intention.push(&plan, Bindings::empty(), trigger);

        // A single step executes the belief formula and, since it's also the last one in the
        // body, reports the frame done immediately - independent of whether the event it just
        // emitted has been drained through `handle_event` yet.
        let result = intention.step(&mut context, &mut knowledge, &mut ());
        assert!(matches!(result, Ok(StepOk::Done)));

        let query_formula = literal_formula("route_active", vec![string("load"), string("c")]);
        assert!(
            (&query_formula)
                .into_query(&knowledge)
                .next_bindings(None)
                .is_some(),
            "belief should already be queryable right after the step that added it"
        );
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct CaptureAction(Variable);

    impl crate::plan::action::Execute for CaptureAction {
        type State = Vec<f32>;
        type UserAction = CaptureAction;

        fn execute<'b, B>(
            self,
            bindings: &B,
            _context: &mut Context<Self::UserAction>,
            _knowledge: &KnowledgeBase,
            state: &mut Self::State,
        ) -> crate::plan::action::ExecuteResult<'b, Self>
        where
            B: crate::bindings::BindingLookup + 'b,
        {
            let value = bindings
                .lookup_as_type::<f32>(&self.0)
                .expect("variable should be bound")
                .expect("bound term should be a number");
            state.push(value);
            crate::plan::action::ExecuteResult::Done(None)
        }
    }

    #[test]
    fn unify_evaluates_ground_arithmetic_and_binds_the_result() {
        let mut intention: Intention<CaptureAction> = Intention::new(0);
        // SAFETY: The environment on the context remains untouched,
        let mut context = unsafe { new_context_without_environment() };
        let mut knowledge = KnowledgeBase::default();
        let mut state = Vec::new();

        let x = variable();
        let trigger = trigger("event", vec![], None);
        let plan = plan(
            trigger.clone(),
            None,
            vec![
                Formula::Unify {
                    lhs: ArithmeticExpression::Term(variable_term(&x)),
                    rhs: ArithmeticExpression::Operation {
                        operator: ArithmeticOperator::Sum,
                        operands: Box::new([
                            ArithmeticExpression::Term(number(2.0)),
                            ArithmeticExpression::Term(number(3.0)),
                        ]),
                    },
                },
                Formula::Action(Action::User(CaptureAction(x))),
            ],
        );

        intention.push(&plan, Bindings::empty(), trigger);

        // step 1: unifies X with the evaluated sum, binding it into the frame.
        let result = intention.step(&mut context, &mut knowledge, &mut state);
        assert!(matches!(result, Ok(StepOk::Pending)));

        // step 2: the capture action reads X back out through the same frame's bindings.
        let result = intention.step(&mut context, &mut knowledge, &mut state);
        assert!(matches!(result, Ok(StepOk::Done)));
        assert_eq!(state, vec![5.0]);
    }

    #[test]
    fn unify_falls_back_to_structural_unification_for_non_arithmetic_terms() {
        let mut intention: Intention<CaptureAction> = Intention::new(0);
        // SAFETY: The environment on the context remains untouched,
        let mut context = unsafe { new_context_without_environment() };
        let mut knowledge = KnowledgeBase::default();
        let mut state = Vec::new();

        let x = variable();
        let trigger = trigger("event", vec![], None);
        let plan = plan(
            trigger.clone(),
            None,
            vec![
                Formula::Unify {
                    lhs: ArithmeticExpression::Term(variable_term(&x)),
                    rhs: ArithmeticExpression::Term(number(42.0)),
                },
                Formula::Action(Action::User(CaptureAction(x))),
            ],
        );

        intention.push(&plan, Bindings::empty(), trigger);
        intention
            .step(&mut context, &mut knowledge, &mut state)
            .ok();
        intention
            .step(&mut context, &mut knowledge, &mut state)
            .ok();

        assert_eq!(state, vec![42.0]);
    }

    #[test]
    fn unify_fails_the_step_when_both_sides_are_ground_and_mismatched() {
        let mut intention: Intention<()> = Intention::new(0);
        // SAFETY: The environment on the context remains untouched,
        let mut context = unsafe { new_context_without_environment() };
        let mut knowledge = KnowledgeBase::default();

        let trigger = trigger("event", vec![], None);
        let plan = plan(
            trigger.clone(),
            None,
            vec![Formula::Unify {
                lhs: ArithmeticExpression::Term(number(2.0)),
                rhs: ArithmeticExpression::Term(number(3.0)),
            }],
        );

        intention.push(&plan, Bindings::empty(), trigger);

        let result = intention.step(&mut context, &mut knowledge, &mut ());
        assert!(matches!(result, Err(StepError::UnifyFailed)));
    }

    #[test]
    fn unify_errors_the_step_when_a_compound_expression_is_only_partially_ground() {
        let mut intention: Intention<()> = Intention::new(0);
        // SAFETY: The environment on the context remains untouched,
        let mut context = unsafe { new_context_without_environment() };
        let mut knowledge = KnowledgeBase::default();

        // Y is bound, Z is free: `(Y+1) + Z` can't fully evaluate, and unlike Jason,
        // ember-bdi has no term shape to unify against the unevaluated expression instead
        // (see plan section 6), so the whole step errors rather than silently succeeding.
        let (y, z) = (variable(), variable());
        let trigger = trigger("event", vec![], None);
        let plan = plan(
            trigger.clone(),
            None,
            vec![Formula::Unify {
                lhs: ArithmeticExpression::Term(number(0.0)),
                rhs: ArithmeticExpression::Operation {
                    operator: ArithmeticOperator::Sum,
                    operands: Box::new([
                        ArithmeticExpression::Operation {
                            operator: ArithmeticOperator::Sum,
                            operands: Box::new([
                                ArithmeticExpression::Term(variable_term(&y)),
                                ArithmeticExpression::Term(number(1.0)),
                            ]),
                        },
                        ArithmeticExpression::Term(variable_term(&z)),
                    ]),
                },
            }],
        );

        intention.push(&plan, bindings(vec![(y, number(2.0).as_view())]), trigger);

        let result = intention.step(&mut context, &mut knowledge, &mut ());
        assert!(matches!(result, Err(StepError::UnifyEvalError(_))));
    }
}
