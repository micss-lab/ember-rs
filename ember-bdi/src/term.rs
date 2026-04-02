use alloc::string::String;
use alloc::vec::Vec;
use core::convert::Infallible;

use bstr::BString;

pub struct Ground(Infallible);

pub struct NonGround(Variable);

pub enum Term<Groundness = NonGround> {
    Number(f32),
    String(BString),
    Variable(Groundness),
    // TODO: Support lists.
    // List(List),
    Structure(Structure<Groundness>),
}

pub struct Variable(String);

pub struct Structure<Groundness = NonGround> {
    functor: Atom,
    arguments: Vec<Term<Groundness>>,
}

pub struct Atom(String);
