use alloc::vec::Vec;

use crate::bindings::Bindings;
use crate::knowledge::store::BeliefBase;
use crate::literal::Literal;
use crate::plan::{Formula, GoalKind, Plan, QueryFormula, Trigger, TriggeringEvent};
use crate::term::view::TermView;
use crate::term::{Atom, NonGround, Structure, Term};
use crate::variable::Variable;

pub fn v() -> Variable {
    Variable::new()
}

pub fn vt(var: &Variable) -> Term {
    Term::Variable(NonGround(var.clone()))
}

pub fn s(str: &str) -> Term {
    Term::String(str.into())
}

pub fn n(num: f32) -> Term {
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

pub fn literal_formula(functor: &str, args: Vec<Term>) -> QueryFormula {
    QueryFormula::Literal(literal(functor, args))
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

pub fn literal_variable(var: &Variable) -> Literal {
    Literal::Variable(NonGround(var.clone()))
}

pub fn bindings<'a>(list: Vec<(Variable, TermView<'a>)>) -> Bindings<'a> {
    let pairs = list
        .into_iter()
        .map(|(v, tv)| (v.id, Some(tv)))
        .collect::<Vec<_>>();
    Bindings::new(pairs, crate::bindings::AliasMap::empty())
}
