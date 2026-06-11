use alloc::format;
use alloc::string::{String, ToString};

use bstr::BString;

use crate::literal::{IntoLiteral, Literal};
use crate::variable::Variable;

use super::owned::Term;
use super::reference::TermRef;
use super::{Ground, NonGround};

pub use ember_bdi_macros::FromTerm;

pub trait FromTerm<'a>: Sized {
    fn from_term(term: TermRef<'a>) -> Result<Self, FromTermError>;
}

#[derive(Debug, Clone)]
pub enum FromTermError {
    InvalidType(
        /// The expected type.
        Option<&'static str>,
    ),
    /// Failure in converting from the term-native type to the more specialized type. For
    /// example, `BString` to `String` when the characters are not all UTF-8.
    IncorrectConversion(ConversionError),
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
                FromTermError::IncorrectConversion(ref e) => e.to_string(),
            }
        )
    }
}

impl core::error::Error for FromTermError {}

#[derive(Debug, Clone)]
pub enum ConversionError {
    InvalidUtf8,
}

impl core::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "coversion error: {}",
            match *self {
                ConversionError::InvalidUtf8 => "invalid utf-8",
            }
        )
    }
}

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
            TermRef::Literal {
                functor, arguments, ..
            } if arguments.is_empty() => Ok(BString::from(functor.0.as_bytes())),
            _ => Err(FromTermError::InvalidType(Some("string or atom"))),
        }
    }
}

impl FromTerm<'_> for String {
    fn from_term(term: TermRef<'_>) -> Result<Self, FromTermError> {
        String::try_from(BString::from_term(term)?)
            .map_err(|_| FromTermError::IncorrectConversion(ConversionError::InvalidUtf8))
    }
}

impl<G> From<f32> for Term<G> {
    fn from(number: f32) -> Self {
        Self::Number(number.into())
    }
}

impl<G> From<String> for Term<G> {
    fn from(string: String) -> Self {
        Self::String(string.into())
    }
}

impl<G> From<BString> for Term<G> {
    fn from(string: BString) -> Self {
        Self::String(string)
    }
}

impl<L> From<L> for Term<Ground>
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

impl From<Variable> for Term<NonGround> {
    fn from(variable: Variable) -> Self {
        Self::Variable(NonGround(variable))
    }
}
