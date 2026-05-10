use alloc::borrow::Cow;
use alloc::collections::BTreeMap;

use crate::term::Term;
use crate::variable::{Variable, VariableId};

#[derive(Debug)]
pub struct Bindings<'a>(BTreeMap<VariableId, Option<Cow<'a, Term>>>);

impl<'a> Bindings<'a> {
    pub(crate) fn new(
        bindings: impl IntoIterator<Item = (VariableId, Option<Cow<'a, Term>>)>,
    ) -> Self {
        Self(bindings.into_iter().collect())
    }
}

#[cfg(test)]
impl<'a> Bindings<'a> {
    pub(crate) fn get(&'a self, variable: &Variable) -> Option<&'a Term> {
        self.0.get(&variable.id)?.as_deref()
    }
}
