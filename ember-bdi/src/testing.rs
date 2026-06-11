use alloc::collections::vec_deque::VecDeque;
use alloc::vec::Vec;
use ember_core::environment::Environment;

use crate::bindings::Bindings;
use crate::context::Context;
use crate::knowledge::store::BeliefBase;
use crate::literal::Literal;
use crate::plan::{Formula, GoalKind, Plan, QueryFormula, Trigger, TriggeringEvent};
use crate::term::view::TermView;
use crate::term::{Atom, NonGround, Structure, Term};
use crate::variable::Variable;

pub fn variable() -> Variable {
    Variable::new()
}

pub fn variable_term(var: &Variable) -> Term {
    Term::Variable(NonGround(var.clone()))
}

pub fn string(str: &str) -> Term {
    Term::String(str.into())
}

pub fn number(num: f32) -> Term {
    Term::Number(num.into())
}
pub fn trigger(functor: &str, args: Vec<Term>, goal: Option<GoalKind>) -> TriggeringEvent {
    TriggeringEvent {
        trigger: Trigger::Addition,
        goal,
        event: Literal::Atom {
            negated: false,
            structure: Structure {
                functor: Atom(functor.into()),
                arguments: if args.is_empty() {
                    None
                } else {
                    Some(args.into_boxed_slice())
                },
            },
        },
    }
}

pub fn literal(functor: &str, args: Vec<Term>) -> Literal {
    Literal::Atom {
        negated: false,
        structure: Structure {
            functor: Atom(functor.into()),
            arguments: if args.is_empty() {
                None
            } else {
                Some(args.into_boxed_slice())
            },
        },
    }
}

pub fn literal_variable(var: &Variable) -> Literal {
    Literal::Variable(NonGround(var.clone()))
}

pub fn literal_formula(functor: &str, args: Vec<Term>) -> QueryFormula {
    QueryFormula::Literal(literal(functor, args))
}

pub fn bindings<'a>(list: Vec<(Variable, TermView<'a>)>) -> Bindings<'a> {
    let pairs = list
        .into_iter()
        .map(|(v, tv)| (v.id, Some(tv)))
        .collect::<Vec<_>>();
    Bindings::new(pairs, crate::bindings::AliasMap::empty())
}

pub fn assert_belief(bb: &mut BeliefBase, functor: &str, args: Vec<Term>) {
    let lit = literal(functor, args)
        .try_into_ground()
        .expect("belief should be ground literal");
    bb.assert_no_event(lit);
}

pub fn plan<A>(
    trigger: TriggeringEvent,
    context: Option<QueryFormula>,
    body: Vec<Formula<A>>,
) -> Plan<A> {
    Plan {
        trigger,
        context,
        body: body.into_boxed_slice(),
    }
}

/// Returns a context for use during testing without an environment initialised. Calling any method
/// that accesses or mutates the environment is undefined behaviour.
pub unsafe fn new_context_without_environment<A>() -> Context<'static, A> {
    let mut environment = Environment::new(VecDeque::with_capacity(0));
    // SAFETY: The context should never be used during testing.
    Context::new(unsafe {
        core::mem::transmute::<&mut Environment, &'static mut Environment>(&mut environment)
    })
}
