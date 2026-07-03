#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodeError {
    EmptyFunctor,
    FunctorContainsNull,
    VariableNameEmpty,
    VariableNameContainsNull,
}

impl core::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "encode error: {}",
            match self {
                Self::EmptyFunctor => "empty functor",
                Self::FunctorContainsNull => "functor contains null",
                Self::VariableNameEmpty => "variable name empty",
                Self::VariableNameContainsNull => "variable name contains null",
            }
        )
    }
}

impl core::error::Error for EncodeError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    ParseFailed,
}

impl core::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "decode error: {}",
            match self {
                Self::ParseFailed => "parse failed",
            }
        )
    }
}

impl core::error::Error for DecodeError {}
