use crate::bindings::TermView;
use crate::variable::VariableId;

#[derive(Debug)]
pub struct BindingConstraint<'a> {
    pub(crate) variable: VariableId,
    pub(crate) value: TermView<'a>,
}

impl<'a> BindingConstraint<'a> {
    pub fn new(variable: VariableId, value: impl Into<TermView<'a>>) -> Self {
        Self {
            variable,
            value: value.into(),
        }
    }
}
