use crate::bindings::{Bindings, TermView};
use crate::variable::VariableId;

use super::error::Result;
use super::solver;

#[derive(Debug)]
pub struct BindingConstraint<'a> {
    pub(super) variable: VariableId,
    pub(super) value: TermView<'a>,
}

impl<'a> BindingConstraint<'a> {
    pub fn new(variable: VariableId, value: impl Into<TermView<'a>>) -> Self {
        Self {
            variable,
            value: value.into(),
        }
    }
}

impl<'a> Bindings<'a> {
    /// Tries to build a unification map of the collected constraints using the existing
    /// bindings as additional constraints.
    ///
    /// # Implementation
    ///
    /// The function does the following: given a collection of constraints, find or create the
    /// partition this variable belongs to. If the partition already contains a value, try to
    /// unify the current value with the new one returning new constraints. Do this for each
    /// constraint in the queue.
    pub(super) fn build_from_constraints<'b>(
        constraints: impl IntoIterator<Item = BindingConstraint<'a>>,
        existing_bindings: Option<&Bindings<'a>>,
    ) -> Result<Self> {
        let mut solver = solver::ConstraintSolver::new(constraints);
        if let Some(existing_bindings) = existing_bindings {
            solver.load_existing_bindings(existing_bindings)?;
        }
        solver.solve()
    }
}
