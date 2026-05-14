use alloc::vec;
use alloc::vec::Vec;

use crate::bindings::{Bindings, StructureView, TermView};
use crate::term::{NonGround, Structure, Term};
use crate::unification::error::UnificationError;
use crate::variable::Variable;

use super::constraint::BindingConstraint;
use super::error::Result;

pub trait Unify<Rhs> {
    /// Collect individual constraints without recursive verification that they are collectively
    /// sound.
    fn collect_constraints<'a>(&'a self, other: Rhs) -> Result<Vec<BindingConstraint<'a>>>
    where
        Rhs: 'a;

    /// Try to unify this structure with something it can be unified with.
    fn unify<'a>(
        &'a self,
        other: Rhs,
        existing_bindings: Option<&Bindings<'a>>,
    ) -> Result<Bindings<'a>>
    where
        Rhs: 'a,
    {
        Bindings::build_from_constraints(self.collect_constraints(other)?, existing_bindings)
    }
}

pub trait UnifyView<'a>
where
    Self: 'a + Sized,
{
    /// Collect individual constraints without recursive verification that they are collectively
    /// sound.
    fn collect_constraints(self, other: Self) -> Result<Vec<BindingConstraint<'a>>>;

    /// Try to unify this structure with something it can be unified with.
    fn unify(self, other: Self, existing_bindings: Option<&Bindings<'a>>) -> Result<Bindings<'a>> {
        Bindings::build_from_constraints(self.collect_constraints(other)?, existing_bindings)
    }
}

impl Unify<&Term> for Term {
    fn collect_constraints<'a>(&'a self, other: &'a Self) -> Result<Vec<BindingConstraint<'a>>>
    where
        Self: 'a,
    {
        use Term::*;

        match (self, other) {
            (Variable(NonGround(v)), t) | (t, Variable(NonGround(v))) => v.collect_constraints(t),

            (Number(n1), Number(n2)) if n1 == n2 => Ok(vec![]),
            (Number(_), Number(_)) => Err(UnificationError::NumberMismatch),

            (String(s1), String(s2)) if s1 == s2 => Ok(vec![]),
            (String(_), String(_)) => Err(UnificationError::StringMismatch),

            (Literal { negated: n1, .. }, Literal { negated: n2, .. }) if n1 != n2 => {
                Err(UnificationError::NegationMismatch)
            }
            (Literal { structure: s1, .. }, Literal { structure: s2, .. }) => {
                s1.collect_constraints(s2)
            }
            _ => Err(UnificationError::TypeMismatch),
        }
    }
}

impl<'a> UnifyView<'a> for TermView<'a> {
    fn collect_constraints(self, other: Self) -> Result<Vec<BindingConstraint<'a>>> {
        match (self, other) {
            (TermView::Term(this), other) | (other, TermView::Term(this)) => {
                this.collect_constraints(other)
            }

            (TermView::Literal { negated: n1, .. }, TermView::Literal { negated: n2, .. })
                if n1 != n2 =>
            {
                Err(UnificationError::NegationMismatch)
            }
            (TermView::Literal { structure: s1, .. }, TermView::Literal { structure: s2, .. }) => {
                s1.collect_constraints(s2)
            }
            (TermView::Number(n1), TermView::Number(n2)) => (n1 == n2)
                .then(alloc::vec::Vec::new)
                .ok_or(UnificationError::NumberMismatch),

            (TermView::Variable(v), other) | (other, TermView::Variable(v)) => {
                v.collect_constraints(other)
            }

            _ => Err(UnificationError::TypeMismatch),
        }
    }
}

impl<'v> Unify<TermView<'v>> for Term {
    fn collect_constraints<'a>(&'a self, other: TermView<'v>) -> Result<Vec<BindingConstraint<'a>>>
    where
        TermView<'v>: 'a,
    {
        match (self, other) {
            (_, TermView::Term(other)) => self.collect_constraints(other),

            (Term::Variable(NonGround(v)), other) => v.collect_constraints(other),
            (other, TermView::Variable(v)) => v.collect_constraints(other),

            (Term::Literal { negated: n1, .. }, TermView::Literal { negated: n2, .. })
                if *n1 != n2 =>
            {
                Err(UnificationError::NegationMismatch)
            }
            (Term::Literal { structure: s1, .. }, TermView::Literal { structure: s2, .. }) => {
                s1.collect_constraints(s2)
            }

            (Term::Number(n1), TermView::Number(n2)) => (*n1 == n2)
                .then(alloc::vec::Vec::new)
                .ok_or(UnificationError::NumberMismatch),

            _ => Err(UnificationError::TypeMismatch),
        }
    }
}

impl Unify<&Structure> for Structure {
    fn collect_constraints<'a>(&'a self, other: &'a Self) -> Result<Vec<BindingConstraint<'a>>>
    where
        Self: 'a,
    {
        if self.functor != other.functor {
            return Err(UnificationError::FunctorMismatch);
        }

        match (&self.arguments, &other.arguments) {
            (Some(args1), Some(args2)) if args1.len() == args2.len() => {
                let mut bindings = Vec::new();
                for (a1, a2) in args1.iter().zip(args2.iter()) {
                    bindings.extend(a1.collect_constraints(a2)?);
                }
                Ok(bindings)
            }
            (None, None) => Ok(vec![]),
            _ => Err(UnificationError::ArityMismatch),
        }
    }
}

impl<'a> UnifyView<'a> for StructureView<'a> {
    fn collect_constraints(self, other: Self) -> Result<Vec<BindingConstraint<'a>>> {
        if self.functor != other.functor {
            return Err(UnificationError::FunctorMismatch);
        }

        match (&self.arguments, &other.arguments) {
            (Some(args1), Some(args2)) if args1.len() == args2.len() => {
                let mut bindings = Vec::new();
                for (a1, a2) in args1.iter().zip(args2.iter()) {
                    bindings.extend(a1.clone().collect_constraints(a2.clone())?);
                }
                Ok(bindings)
            }
            (None, None) => Ok(vec![]),
            _ => Err(UnificationError::ArityMismatch),
        }
    }
}

impl<'v> Unify<StructureView<'v>> for Structure {
    fn collect_constraints<'a>(
        &'a self,
        other: StructureView<'v>,
    ) -> Result<Vec<BindingConstraint<'a>>>
    where
        StructureView<'v>: 'a,
    {
        if &self.functor != other.functor {
            return Err(UnificationError::FunctorMismatch);
        }

        match (&self.arguments, &other.arguments) {
            (Some(args1), Some(args2)) if args1.len() == args2.len() => {
                let mut bindings = Vec::new();
                for (a1, a2) in args1.iter().zip(args2.iter()) {
                    bindings.extend(a1.collect_constraints(a2.clone())?);
                }
                Ok(bindings)
            }
            (None, None) => Ok(vec![]),
            _ => Err(UnificationError::ArityMismatch),
        }
    }
}

impl Unify<&Term> for Variable {
    fn collect_constraints<'a>(&'a self, other: &'a Term) -> Result<Vec<BindingConstraint<'a>>>
    where
        Term: 'a,
    {
        // TODO: Check that the term can be converted to the type of the variable.

        Ok(vec![BindingConstraint::new(self.id, other)])
    }
}

impl<'v> Unify<TermView<'v>> for Variable {
    fn collect_constraints<'a>(&'a self, other: TermView<'v>) -> Result<Vec<BindingConstraint<'a>>>
    where
        TermView<'v>: 'a,
    {
        // TODO: Check that the term can be converted to the type of the variable.

        Ok(vec![BindingConstraint::new(self.id, other.clone())])
    }
}
