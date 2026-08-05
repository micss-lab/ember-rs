use crate::resolve::ResolveFailure;

pub(crate) type Result = ::core::result::Result<StepOk, StepError>;

#[derive(Debug)]
pub(crate) enum StepOk {
    Pending,
    Done,
}

impl StepOk {
    pub(crate) fn done() -> Result {
        Ok(Self::Done)
    }

    pub(crate) fn pending() -> Result {
        Ok(Self::Pending)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum StepError {
    ResolveFailure(ResolveFailure),
    ResolveIncomplete,
}

impl core::fmt::Display for StepError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use StepError::*;
        use alloc::string::ToString;
        write!(
            f,
            "frame step error: {}",
            match self {
                ResolveFailure(failure) => failure.to_string(),
                ResolveIncomplete => "resolve incomplete".to_string(),
            }
        )
    }
}

impl From<ResolveFailure> for StepError {
    fn from(error: ResolveFailure) -> Self {
        Self::ResolveFailure(error)
    }
}
