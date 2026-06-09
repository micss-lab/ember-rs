use alloc::format;

use bstr::BString;

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
