#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use alloc::string::{String, ToString};

use bstr::BString;
use ember_util::cmp::TotalCmpF32;

pub mod binary;
pub mod error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BdilContent {
    Literal(Literal),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Literal {
    pub negated: bool,
    pub functor: Functor,
    pub arguments: Option<Box<[Term]>>,
}

impl From<Literal> for BdilContent {
    fn from(literal: Literal) -> Self {
        BdilContent::Literal(literal)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Functor(pub String);

impl<T: ToString> From<T> for Functor {
    fn from(s: T) -> Self {
        Self(s.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Term {
    Int(i32),
    Float(TotalCmpF32),
    String(BString),
    Literal(Literal),
    Variable(Variable),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Variable {
    pub name: String,
}

impl<T: Into<String>> From<T> for Variable {
    fn from(name: T) -> Self {
        Self { name: name.into() }
    }
}
