use crate::bindings::resolver::ResolveFailure;
use crate::bindings::{BindingLookup, OwnedBindings};
use crate::term::view::TermView;
use crate::variable::Variable;

#[derive(Debug)]
pub(crate) enum ReadOnlyBindings<'a> {
    Owned(OwnedBindings),
    Borrowed(&'a OwnedBindings),
}

impl<'a> BindingLookup for ReadOnlyBindings<'a> {
    fn lookup_view<'b>(&'b self, variable: &Variable) -> Option<TermView<'b>> {
        match self {
            ReadOnlyBindings::Owned(bindings) => bindings.lookup_view(variable),
            ReadOnlyBindings::Borrowed(bindings) => bindings.lookup_view(variable),
        }
    }
}

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
