pub(crate) type Result<T> = core::result::Result<T, UnificationError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnificationError {
    NumberMismatch,
    StringMismatch,
    FunctorMismatch,
    ArityMismatch,
    TypeMismatch,
    NegationMismatch,
    CyclicReference,
}

impl core::fmt::Display for UnificationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "unification failed: ")?;
        match self {
            Self::NumberMismatch => write!(f, "number mismatch"),
            Self::StringMismatch => write!(f, "string mismatch"),
            Self::FunctorMismatch => write!(f, "functor mismatch"),
            Self::ArityMismatch => write!(f, "arity mismatch"),
            Self::TypeMismatch => write!(f, "type mismatch"),
            Self::NegationMismatch => write!(f, "negation mismatch"),
            Self::CyclicReference => write!(f, "cyclic reference detected"),
        }
    }
}

impl core::error::Error for UnificationError {}
