use core::convert::Infallible;

use crate::variable::Variable;

pub use self::owned::{Atom, Structure, Term, TotalCmpF32};

pub mod owned;
pub mod view;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ground(pub(crate) Infallible);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NonGround(pub Variable);
