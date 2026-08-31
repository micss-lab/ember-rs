use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::collections::vec_deque::VecDeque;
use alloc::rc::Rc;
use alloc::vec::Vec;

use ember_core::environment::Environment;

use crate::bindings::Bindings;
use crate::context::{Context, PureContext};
use crate::knowledge::base::KnowledgeBase;
use crate::literal::Literal;
use crate::plan::action::{Execute, ExecuteResult};
use crate::plan::{Formula, GoalKind, Plan, QueryFormula, Trigger, TriggeringEvent};
use crate::term::owned::composite::VariableOrList;
use crate::term::view::TermView;
use crate::term::{Atom, Structure, Term};
use crate::variable::Variable;

impl Execute for () {
    type State = ();
    type UserAction = ();

    fn execute<'b>(
        self,
        _bindings: &Bindings<'b>,
        _context: &mut Context<Self::UserAction>,
        _knowledge: &KnowledgeBase,
        _state: &mut Self::State,
    ) -> ExecuteResult<'b, Self> {
        ExecuteResult::Done(None)
    }
}

pub fn variable() -> Variable {
    Variable::new()
}

pub fn variable_term(var: &Variable) -> Term {
    Term::Variable(var.clone())
}

pub fn string(str: &str) -> Term {
    Term::String(str.into())
}

pub fn number(num: f32) -> Term {
    Term::Number(num.into())
}

pub fn list(items: Vec<Term>) -> Term {
    Term::List(items.into_boxed_slice())
}

pub fn variable_or_list(items: Vec<Term>) -> VariableOrList {
    VariableOrList::List(items.into_boxed_slice())
}
pub fn trigger(functor: &str, args: Vec<Term>, goal: Option<GoalKind>) -> TriggeringEvent {
    TriggeringEvent {
        trigger: Trigger::Addition,
        goal,
        event: Literal {
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
    Literal {
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

pub fn bindings<'a>(list: Vec<(Variable, TermView<'a>)>) -> Bindings<'a> {
    let pairs = list
        .into_iter()
        .map(|(v, tv)| (v.id, Some(tv)))
        .collect::<Vec<_>>();
    Bindings::new(pairs, crate::bindings::AliasMap::empty())
}

pub fn assert_belief(bb: &mut KnowledgeBase, functor: &str, args: Vec<Term>) {
    let lit = literal(functor, args);
    bb.assert_no_event(lit);
}

/// The `PureContext` used by [`new_context_without_environment`] and
/// [`new_context_with_environment`], for tests that need to query a `PureAction::Me` leaf.
pub fn pure_context() -> PureContext {
    PureContext::new(Rc::new(Cow::Borrowed("test-agent")))
}

pub fn plan<A>(
    trigger: TriggeringEvent,
    context: Option<QueryFormula>,
    body: impl IntoIterator<Item = Formula<A>>,
) -> Plan<A> {
    Plan {
        trigger,
        context,
        body: body.into_iter().collect(),
    }
}

/// Returns a context for use during testing without an environment initialised. Calling any method
/// that accesses or mutates the environment is undefined behaviour.
pub unsafe fn new_context_without_environment<A>() -> Context<'static, A> {
    let mut environment = Environment::new(VecDeque::with_capacity(0));
    // SAFETY: The context should never be used during testing.
    Context::new(pure_context(), unsafe {
        core::mem::transmute::<&mut Environment, &'static mut Environment>(&mut environment)
    })
}

/// Returns a context for use during testing backed by a real (leaked) environment, safe to read
/// from and write to. Use this instead of [`new_context_without_environment`] for actions that
/// touch the environment, e.g. `.stop_platform` or `.send`.
pub fn new_context_with_environment<A>() -> Context<'static, A> {
    let environment: &'static mut Environment =
        Box::leak(Box::new(Environment::new(VecDeque::with_capacity(0))));
    Context::new(pure_context(), environment)
}
