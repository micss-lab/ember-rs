#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;

use bstr::BString;
use ember_util::cmp::TotalCmpF32;

pub mod binary;
pub mod error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Literal {
    pub negated: bool,
    pub functor: Functor,
    pub arguments: Option<Box<[Term]>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Functor(pub String);

impl<T: Into<String>> From<T> for Functor {
    fn from(s: T) -> Self {
        Self(s.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Term {
    Int(i32),
    Float(TotalCmpF32),
    Str(BString),
    Literal(Literal),
    Variable(Variable),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Variable {
    pub name: String,
}

impl<T: Into<String>> From<T> for Variable {
    fn from(name: T) -> Self {
        Self { name: name.into() }
    }
}
