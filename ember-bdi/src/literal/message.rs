use alloc::collections::BTreeMap;

use ember_core::message::content::ember_bdil::{
    Literal as MessageLiteral, Variable as MessageVariable,
};

use crate::{
    term::{Atom, Structure, Term},
    variable::Variable,
};

use super::Literal;

impl From<MessageLiteral> for Literal {
    fn from(literal: MessageLiteral) -> Self {
        let mut variable_map = BTreeMap::new();
        Self::from_message_literal(literal, &mut variable_map)
    }
}

impl From<Literal> for MessageLiteral {
    fn from(literal: Literal) -> Self {
        let mut variable_map = BTreeMap::new();
        literal.into_message_literal(&mut variable_map)
    }
}

impl Literal {
    pub(crate) fn from_message_literal(
        MessageLiteral {
            negated,
            functor,
            arguments,
        }: MessageLiteral,
        variable_map: &mut BTreeMap<MessageVariable, Variable>,
    ) -> Self {
        Self {
            negated,
            structure: Structure {
                functor: Atom::from_message_functor(functor),
                arguments: arguments.map(|args| {
                    args.into_iter()
                        .map(|t| Term::from_message_term(t, variable_map))
                        .collect()
                }),
            },
        }
    }

    pub(crate) fn into_message_literal(
        self,
        variable_map: &mut BTreeMap<Variable, MessageVariable>,
    ) -> MessageLiteral {
        let Self {
            negated,
            structure: Structure { functor, arguments },
        } = self;
        MessageLiteral {
            negated,
            functor: functor.into_message_functor(),
            arguments: arguments.map(|args| {
                args.into_iter()
                    .map(|t| t.into_message_term(variable_map))
                    .collect()
            }),
        }
    }
}
