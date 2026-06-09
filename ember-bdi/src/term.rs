use core::convert::Infallible;

use crate::variable::Variable;

pub use self::from::{FromTerm, FromTermError};
pub use self::owned::{Atom, Structure, Term, TotalCmpF32};

pub mod from;
pub mod owned;
pub mod reference;
pub(crate) mod view;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ground(pub(crate) Infallible);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NonGround(pub Variable);
