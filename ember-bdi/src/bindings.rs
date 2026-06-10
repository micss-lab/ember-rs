use core::marker::PhantomData;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::term::Term;
use crate::term::conversion::{FromTerm, FromTermError};
use crate::term::reference::TermRef;
use crate::term::view::{StructureView, TermView};
use crate::unification::constraint::BindingConstraint;
use crate::unification::error::UnificationError;
use crate::variable::{Variable, VariableId};

pub(crate) mod solver;

#[derive(Debug, Clone, Default)]
pub struct Bindings<'a, T = TermView<'a>> {
    pub(crate) bindings: Option<BTreeMap<VariableId, Option<T>>>,
    pub(crate) aliases: AliasMap,
    lifetime_: PhantomData<&'a ()>,
}

impl<'a, T> Bindings<'a, T> {
    pub(crate) fn empty() -> Self {
        Self {
            bindings: None,
            aliases: AliasMap::empty(),
            lifetime_: PhantomData,
        }
    }
}

impl<'a> Bindings<'a, TermView<'a>> {
    pub(crate) fn new(
        bindings: impl IntoIterator<Item = (VariableId, Option<TermView<'a>>)>,
        aliases: AliasMap,
    ) -> Self {
        Self {
            bindings: Some(bindings.into_iter().collect()),
            aliases,
            lifetime_: PhantomData,
        }
    }
}

impl<'a> Bindings<'a, TermView<'a>> {
    pub(crate) fn get_view(&self, variable: &Variable) -> Option<&TermView<'a>> {
        self.bindings.as_ref()?.get(&variable.id)?.as_ref()
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
    pub(crate) fn build_from_constraints<'b>(
        constraints: impl IntoIterator<Item = BindingConstraint<'a>>,
        existing_bindings: Option<&Bindings<'a>>,
    ) -> Result<Self, UnificationError> {
        let mut solver = solver::ConstraintSolver::new(constraints);
        if let Some(existing_bindings) = existing_bindings {
            solver.load_existing_bindings(existing_bindings)?;
        }
        solver.solve()
    }
}

pub type OwnedBindings = Bindings<'static, Term>;

impl From<Bindings<'_>> for OwnedBindings {
    fn from(
        Bindings {
            bindings, aliases, ..
        }: Bindings<'_>,
    ) -> Self {
        Self {
            bindings: bindings.map(|b| {
                b.into_iter()
                    .map(|(k, v)| (k, v.map(|v| v.to_owned())))
                    .collect()
            }),
            aliases,
            lifetime_: PhantomData,
        }
    }
}

impl OwnedBindings {
    pub(crate) fn merge<const N: usize>(mut bindings: [Self; N]) -> Result<Self, UnificationError> {
        let mut solver = solver::ConstraintSolver::new(core::iter::empty());
        bindings.iter_mut().try_for_each(|b| {
            if let Some(bindings) = &b.bindings {
                solver.register_constraints(
                    bindings
                        .iter()
                        .filter_map(|(v, t)| t.as_ref().map(|t| (*v, t.as_view()))),
                )?;
            }

            solver.register_aliases(core::mem::replace(&mut b.aliases.0, Vec::with_capacity(0)))
        })?;
        Ok(solver.solve()?.into())
    }
}

pub trait BindingLookup {
    fn lookup_view<'a>(&'a self, variable: &Variable) -> Option<TermView<'a>>;

    /// Lookup the term bound to the given variable.
    fn lookup<'a>(&'a self, variable: &Variable) -> Option<TermRef<'a>> {
        self.lookup_view(variable).map(Into::into)
    }

    /// Loopup the term bound to the give variable and parse the term into the required type.
    fn lookup_as_type<'a, T>(&'a self, variable: &Variable) -> Option<Result<T, FromTermError>>
    where
        T: FromTerm<'a>,
    {
        self.lookup(variable).map(T::from_term)
    }
}

impl BindingLookup for Bindings<'_> {
    fn lookup_view<'a>(&'a self, variable: &Variable) -> Option<TermView<'a>> {
        self.get_view(variable).cloned()
    }
}

impl BindingLookup for OwnedBindings {
    fn lookup_view<'a>(&'a self, variable: &Variable) -> Option<TermView<'a>> {
        self.bindings
            .as_ref()?
            .get(&variable.id)?
            .as_ref()
            .map(|t| t.as_view())
    }
}

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
}
