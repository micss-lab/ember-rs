use alloc::format;
use alloc::string::{String, ToString};

use bstr::{BString, ByteSlice};

use ember_core::agent::Aid;

use crate::literal::Literal;
use crate::variable::Variable;

use super::owned::Term;
use super::reference::TermRef;

pub use ember_bdi_macros::FromTerm;

pub trait FromTerm<'a>: Sized {
    fn from_term(term: TermRef<'a>) -> Result<Self, FromTermError>;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
                    Some(expected) => format!("invalid type: expected {expected}"),
                    None => "invalid type".into(),
                },
                FromTermError::IncorrectConversion(ref e) => e.to_string(),
            }
        )
    }
}

impl core::error::Error for FromTermError {}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConversionError {
    InvalidUtf8,
    InvalidAid(
        /// The error thrown by parsing the aid.
        String,
    ),
}

impl core::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "coversion error: {}",
            match *self {
                ConversionError::InvalidUtf8 => "invalid utf-8".to_string(),
                ConversionError::InvalidAid(ref e) => format!("invalid aid: {e}"),
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
                functor,
                arguments,
                negated,
            } if arguments.is_empty() && !negated => Ok(BString::from(functor.0.as_bytes())),
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

impl FromTerm<'_> for Aid {
    fn from_term(term: TermRef<'_>) -> Result<Self, FromTermError> {
        match term {
            TermRef::String(s) => s
                .to_str()
                .map_err(|_| ConversionError::InvalidUtf8)
                .and_then(|s| s.parse().map_err(ConversionError::InvalidAid))
                .map_err(FromTermError::IncorrectConversion),
            TermRef::Literal {
                negated,
                functor,
                arguments,
            } => Err(FromTermError::InvalidType(Some(
                "string (TODO: Implement literal support)",
            ))),
            _ => Err(FromTermError::InvalidType(Some(
                "aid string <agent-name@platform-address>",
            ))),
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

impl From<Literal> for Term {
    fn from(literal: Literal) -> Self {
        Self::Literal(literal)
    }
}

impl From<Variable> for Term {
    fn from(variable: Variable) -> Self {
        Self::Variable(variable)
    }
}
