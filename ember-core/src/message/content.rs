use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::marker::PhantomData;

use chrono::{DateTime, FixedOffset};

pub mod lang;

/// List of expressions to form the content of an ACL message.
#[derive(Debug, Clone, PartialEq)]
pub struct Content(pub Vec<ContentElement>);

impl Content {
    pub fn try_from_sl(input: impl AsRef<bstr::BStr>) -> Result<Self, String> {
        lang::sl::sl0_parser::content(&crate::util::parsing::BStr::from(input.as_ref()))
            .map_err(|e| e.to_string())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContentElement {
    AgentAction(AgentAction),
    Predicate(Predicate),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Concept {
    /// Type defining the concept.
    pub symbol: bstr::BString,
    /// Parameters belonging to the concept.
    pub parameters: ConceptParameters,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AgentAction {
    /// Agent performing the action.
    pub agent: Term,
    /// The action to be performed.
    pub action: Term,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Predicate {
    Regular {
        symbol: bstr::BString,
        terms: Vec<Term>,
    },
    Result {
        lhs: Term,
        rhs: Term,
    },
    Done {
        action: AgentAction,
    },
    Bool(bool),
}

/// Recursive structure defining the concept of a term.
#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    Constant(Constant),
    Set(Set),
    Sequence(Seq),
    Concept(Concept),
    Action(Box<AgentAction>),
}

/// Parameters part of a functional term.
#[derive(Debug, Clone, PartialEq)]
pub enum ConceptParameters {
    Positional(Vec<Term>),
    ByName(Vec<(bstr::BString, Term)>),
}

impl ConceptParameters {
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Positional(p) => p.len(),
            Self::ByName(p) => p.len(),
        }
    }
}

/// Numerical, string, and time-related constants.
#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Number(Number),
    String(bstr::BString),
    Datatime(DateTime<FixedOffset>),
}

/// Numerical constant.
#[derive(Debug, Clone, PartialEq)]
pub enum Number {
    Int(i32),
    Float(f32),
}

pub type Set = Collection<collection::SetLike>;
pub type Seq = Collection<collection::SeqLike>;

/// General collection type.
///
/// Note: A set cannot be stored in a [`BTreeSet`] as it requires the
/// items to be [`Ord`]. They cannot be as this would require evaluating
/// the terms before storing them.
///
/// [`BTreeSet`]: alloc::collections::btree_set::BTreeSet
/// [`Ord`]: core::cmp::Ord
#[derive(Debug, Clone, PartialEq)]
pub struct Collection<C> {
    /// Items in the collection.
    items: Vec<Term>,
    /// Semantics behind the collection.
    _marker: PhantomData<C>,
}
mod collection {
    #[derive(Debug, Clone, PartialEq)]
    pub struct SetLike;
    #[derive(Debug, Clone, PartialEq)]
    pub struct SeqLike;
}

impl<C> core::ops::Deref for Collection<C> {
    type Target = [Term];

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl<C> FromIterator<Term> for Collection<C> {
    fn from_iter<T: IntoIterator<Item = Term>>(iter: T) -> Self {
        Self {
            items: iter.into_iter().collect(),
            _marker: PhantomData,
        }
    }
}
