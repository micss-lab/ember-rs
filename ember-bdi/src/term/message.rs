use alloc::collections::btree_map::BTreeMap;
use ember_core::message::content::ember_bdil::{
    Functor, Term as MessageTerm, Variable as MessageVariable,
};

use crate::variable::Variable;

use super::{Atom, Term};

impl Term {
    pub(crate) fn from_message_term(
        term: MessageTerm,
        variable_map: &mut BTreeMap<MessageVariable, Variable>,
    ) -> Self {
        match term {
            MessageTerm::Int(i) => {
                log::warn!("Converting between ints and floats is not recommended.");
                Self::Number((i as f32).into())
            }
            MessageTerm::Float(f) => Self::Number(f),
            MessageTerm::String(s) => Self::String(s),
            MessageTerm::Literal(l) => Self::Literal(l.into()),
            MessageTerm::Variable(v) => {
                Term::Variable(variable_map.entry(v).or_insert_with(Variable::new).clone())
            }
        }
    }

    pub(crate) fn into_message_term(
        self,
        variable_map: &mut BTreeMap<Variable, MessageVariable>,
    ) -> MessageTerm {
        match self {
            Term::Number(n) => MessageTerm::Float(n),
            Term::String(s) => MessageTerm::String(s),
            Term::Literal(l) => MessageTerm::Literal(l.into_message_literal(variable_map)),
            Term::Variable(variable) => MessageTerm::Variable(
                variable_map
                    .entry(variable.clone())
                    .or_insert_with(|| variable.into_message_variable())
                    .clone(),
            ),
        }
    }
}

impl Atom {
    pub(crate) fn from_message_functor(Functor(functor): Functor) -> Self {
        Self(functor)
    }

    pub(crate) fn into_message_functor(self) -> Functor {
        let Self(functor) = self;
        Functor(functor)
    }
}
