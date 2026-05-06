use alloc::string::String;
use alloc::vec::Vec;
use core::convert::Infallible;

use bstr::BString;

use crate::variable::Variable;

mod unification;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ground(Infallible);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NonGround(pub Variable);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Term<Groundness = NonGround> {
    Number(TotalCmpF32),
    String(BString),
    Variable(Groundness),
    // TODO: Support lists.
    // List(List),
    Literal {
        /// Explicit negation, not the closed-wold principle form of negation.
        negated: bool,
        structure: Structure<Groundness>,
    },
}

impl Term<Ground> {
    pub fn into_non_ground(self) -> Term<NonGround> {
        use Term::*;
        match self {
            Number(n) => Number(n),
            String(s) => String(s),
            Variable(Ground(i)) => {
                // As the type of I is `Infallible` which is an enum with zero
                // variants, this is the best way to ensure that a change to this type
                // would result in a compiler error down the line. Using `unreachable!`
                // would not trigger an error if the type ever changes.
                match i {}
            }
            Literal {
                negated,
                structure: s,
            } => Literal {
                negated,
                structure: s.into_non_ground(),
            },
        }
    }
}

impl Term<NonGround> {
    pub fn try_into_ground(self) -> Option<Term<Ground>> {
        use Term::*;
        Some(match self {
            Number(n) => Number(n),
            String(s) => String(s),
            Variable(_) => return None,
            Literal {
                negated,
                structure: s,
            } => Literal {
                negated,
                structure: s.try_into_ground()?,
            },
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TotalCmpF32(f32);

impl From<f32> for TotalCmpF32 {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl core::ops::Deref for TotalCmpF32 {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for TotalCmpF32 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl PartialOrd for TotalCmpF32 {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TotalCmpF32 {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl PartialEq for TotalCmpF32 {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for TotalCmpF32 {}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Structure<Groundness = NonGround> {
    pub functor: Atom,
    pub arguments: Option<Vec<Term<Groundness>>>,
}

impl Structure<Ground> {
    pub fn into_non_ground(self) -> Structure<NonGround> {
        let Structure { functor, arguments } = self;
        Structure {
            functor,
            arguments: arguments.map(|a| a.into_iter().map(|t| t.into_non_ground()).collect()),
        }
    }
}

impl Structure<NonGround> {
    pub fn try_into_ground(self) -> Option<Structure<Ground>> {
        let Structure { functor, arguments } = self;
        Some(Structure {
            functor,
            arguments: match arguments {
                Some(a) => Some(
                    a.into_iter()
                        .map(|t| t.try_into_ground())
                        .collect::<Option<Vec<_>>>()?,
                ),
                None => None,
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Atom(pub String);
