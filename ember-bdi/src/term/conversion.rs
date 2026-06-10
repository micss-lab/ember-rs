use alloc::format;
use alloc::string::String;

use bstr::BString;

use crate::literal::{IntoLiteral, Literal};
use crate::variable::Variable;

use super::NonGround;
use super::owned::Term;
use super::reference::TermRef;

pub trait FromTerm<'a>: Sized {
    fn from_term(term: TermRef<'a>) -> Result<Self, FromTermError>;
}

#[derive(Debug, Clone)]
pub enum FromTermError {
    InvalidType(
        /// The expected type.
        Option<&'static str>,
    ),
}

impl core::fmt::Display for FromTermError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "from term error: {}",
            match *self {
                FromTermError::InvalidType(expected) => match expected {
                    Some(expected) => format!("invalid type: expected {}", expected),
                    None => "invalid type".into(),
                },
            }
        )
    }
}

impl core::error::Error for FromTermError {}

impl FromTerm<'_> for f32 {
    fn from_term(term: TermRef<'_>) -> Result<Self, FromTermError> {
        match term {
            TermRef::Number(n) => Ok(*n),
            _ => Err(FromTermError::InvalidType(Some("number"))),
        }
    }
}

impl FromTerm<'_> for BString {
    fn from_term(term: TermRef<'_>) -> Result<Self, FromTermError> {
        match term {
            TermRef::String(s) => Ok(s.clone()),
            _ => Err(FromTermError::InvalidType(Some("string"))),
        }
    }
}

impl From<f32> for Term {
    fn from(number: f32) -> Self {
        Self::Number(number.into())
    }
}

impl From<String> for Term {
    fn from(string: String) -> Self {
        Self::String(string.into())
    }
}

impl From<BString> for Term {
    fn from(string: BString) -> Self {
        Self::String(string)
    }
}

impl<L> From<L> for Term
where
    L: IntoLiteral,
{
    fn from(literal: L) -> Self {
        match literal.into_literal() {
            Literal::Atom { negated, structure } => Self::Literal { negated, structure },
            Literal::Variable(v) => Self::Variable(v),
        }
    }
}

impl From<Variable> for Term {
    fn from(variable: Variable) -> Self {
        Self::Variable(NonGround(variable))
    }
}
