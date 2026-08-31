use alloc::vec::Vec;

use ember_collections::{SmallMap, SmallSet};

use crate::term::conversion::{FromTerm, FromTermError};
use crate::term::reference::TermRef;
use crate::term::view::{StructureView, TermView};
use crate::unification::constraint::BindingConstraint;
use crate::unification::error::UnificationError;
use crate::variable::{Variable, VariableId};

pub(crate) mod solver;

#[derive(Debug, Clone, Default)]
pub struct Bindings<'a> {
    pub(crate) bindings: Option<SmallMap<VariableId, Option<TermView<'a>>>>,
    pub(crate) aliases: AliasMap,
}

impl<'a> Bindings<'a> {
    pub(crate) fn empty() -> Self {
        Self {
            bindings: None,
            aliases: AliasMap::empty(),
        }
    }

    pub(crate) fn new(
        bindings: impl IntoIterator<Item = (VariableId, Option<TermView<'a>>)>,
        aliases: AliasMap,
    ) -> Self {
        Self {
            bindings: Some(bindings.into_iter().collect()),
            aliases,
        }
    }

    /// Filters the bound variables and only retains those present in the specified set.
    pub(crate) fn retain_variables(&mut self, variables: &SmallSet<VariableId>) {
        if let Some(b) = self.bindings.as_mut() {
            b.retain(|v, _| variables.contains(v))
        }
        self.aliases.retain_variables(variables);
    }

    pub(crate) fn get_view(&self, variable: &Variable) -> Option<&TermView<'a>> {
        self.bindings.as_ref()?.get(&variable.id)?.as_ref()
    }

    /// Lookup the given variable as a view.
    pub(crate) fn lookup_view(&self, variable: &Variable) -> Option<TermView<'a>> {
        self.get_view(variable).cloned()
    }

    /// Lookup the given variable as a reference to the stored view.
    pub(crate) fn lookup(&self, variable: &Variable) -> Option<TermRef<'_>> {
        Some(self.get_view(variable)?.into())
    }

    /// Loopup the term bound to the given variable and parse the term into the required type.
    pub(crate) fn lookup_as_type<T>(&self, variable: &Variable) -> Option<Result<T, FromTermError>>
    where
        T: for<'s> FromTerm<'s>,
    {
        self.lookup(variable).map(T::from_term)
    }

    /// Tries to build a unification map of the collected constraints using the existing
    /// bindings as additional constraints.
    ///
    /// # Implementation
    ///
    /// The function does the following: given a collection of constraints, find or create the
    /// partition this variable belongs to. If the partition already contains a value, try to
    /// unify the current value with the new one returning new constraints. Do this for each
    /// constraint in the queue.
    pub(crate) fn build_from_constraints(
        constraints: impl IntoIterator<Item = BindingConstraint<'a>>,
        existing_bindings: Option<&Bindings<'a>>,
    ) -> Result<Self, UnificationError> {
        let mut solver = solver::ConstraintSolver::new(constraints);
        if let Some(existing_bindings) = existing_bindings {
            solver.load_existing_bindings(existing_bindings)?;
        }
        solver.solve()
    }

    pub(crate) fn merge_views<'b>(
        bindings: impl IntoIterator<Item = &'b Self>,
    ) -> Result<Self, UnificationError>
    where
        'a: 'b,
    {
        let mut solver = solver::ConstraintSolver::new(core::iter::empty());
        for b in bindings {
            solver.load_existing_bindings(b)?;
        }
        solver.solve()
    }

    pub(crate) fn merge<const N: usize>(mut bindings: [Self; N]) -> Result<Self, UnificationError> {
        let mut solver = solver::ConstraintSolver::new(core::iter::empty());
        bindings.iter_mut().try_for_each(|b| {
            if let Some(bindings) = &b.bindings {
                solver.register_constraints(
                    bindings
                        .iter()
                        .filter_map(|(v, t)| t.as_ref().map(|t| (*v, t.clone()))),
                )?;
            }

            solver.register_aliases(core::mem::replace(&mut b.aliases.0, Vec::with_capacity(0)))
        })?;
        solver.solve()
    }

    /// Widens every bound view to `'static`, detaching the bindings from whatever they
    /// currently borrow. Callers that need to persist bindings past the lifetime of what
    /// produced them (e.g. a frame storing them across ticks) call this once, at the point of
    /// storage - not something `Bindings` forces on every lookup.
    pub(crate) fn into_owned(self) -> OwnedBindings {
        Bindings {
            bindings: self.bindings.map(|b| {
                b.into_iter()
                    .map(|(k, v)| (k, v.map(|v| v.to_owned_view())))
                    .collect()
            }),
            aliases: self.aliases,
        }
    }
}

pub type OwnedBindings = Bindings<'static>;

#[derive(Debug, Clone, Default)]
pub(crate) struct AliasMap(Vec<(VariableId, VariableId)>);

impl AliasMap {
    pub(crate) fn new(aliases: impl IntoIterator<Item = (VariableId, VariableId)>) -> Self {
        Self(aliases.into_iter().collect())
    }

    pub(crate) fn empty() -> Self {
        Self(Vec::with_capacity(0))
    }

    pub(crate) fn iter(&self) -> core::slice::Iter<'_, (VariableId, VariableId)> {
        self.0.iter()
    }

    fn retain_variables(&mut self, variables: &SmallSet<VariableId>) {
        self.0
            .retain(|(v1, v2)| variables.contains(v1) && variables.contains(v2));
    }
}
