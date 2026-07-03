use crate::term::view::TermView;
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

impl<'a, T> From<(VariableId, T)> for BindingConstraint<'a>
where
    T: Into<TermView<'a>>,
{
    fn from((variable, term): (VariableId, T)) -> Self {
        Self::new(variable, term)
    }
}
