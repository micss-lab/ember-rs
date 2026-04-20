use alloc::string::String;
use alloc::vec::Vec;
use core::convert::Infallible;

use bstr::BString;

use crate::variable::Variable;

pub struct Ground(Infallible);

pub struct NonGround(pub Variable);

pub enum Term<Groundness = NonGround> {
    Number(f32),
    String(BString),
    Variable(Groundness),
    // TODO: Support lists.
    // List(List),
    Structure(Structure<Groundness>),
}

pub struct Structure<Groundness = NonGround> {
    pub functor: Atom,
    pub arguments: Option<Vec<Term<Groundness>>>,
}

pub struct Atom(pub String);
