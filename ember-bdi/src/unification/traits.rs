use alloc::vec;
use alloc::vec::Vec;

use bstr::ByteSlice;

use crate::bindings::Bindings;
use crate::literal::{Literal, LiteralView};
use crate::term::view::{StructureView, TermView};
use crate::term::{Structure, Term};
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
            (Variable(v), t) | (t, Variable(v)) => v.collect_constraints(t),

            (Number(n1), Number(n2)) if n1 == n2 => Ok(vec![]),
            (Number(_), Number(_)) => Err(UnificationError::NumberMismatch),

            (String(s1), String(s2)) if s1 == s2 => Ok(vec![]),
            (String(_), String(_)) => Err(UnificationError::StringMismatch),

            (Literal(l1), Literal(l2)) => l1.collect_constraints(l2),

            // Lists only unify when both sides have equal length, matching element-wise (same
            // failure mode as `Structure`'s argument list).
            //
            // TODO: Support Prolog-style partial-list unification via the cons operator
            // (`'.'/2`, i.e. `[Head|Tail]`) so lists of different lengths can unify by binding a
            // tail variable.
            (List(l1), List(l2)) if l1.len() == l2.len() => {
                let mut constraints = Vec::new();
                for (a1, a2) in l1.iter().zip(l2.iter()) {
                    constraints.extend(a1.collect_constraints(a2)?);
                }
                Ok(constraints)
            }
            (List(_), List(_)) => Err(UnificationError::ArityMismatch),

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

            (
                TermView::Literal(LiteralView { negated: n1, .. }),
                TermView::Literal(LiteralView { negated: n2, .. }),
            ) if n1 != n2 => Err(UnificationError::NegationMismatch),
            (
                TermView::Literal(LiteralView { structure: s1, .. }),
                TermView::Literal(LiteralView { structure: s2, .. }),
            ) => s1.collect_constraints(s2),
            (TermView::Number(n1), TermView::Number(n2)) => (n1 == n2)
                .then(alloc::vec::Vec::new)
                .ok_or(UnificationError::NumberMismatch),
            (TermView::String(a), TermView::String(b)) => (a == b)
                .then(alloc::vec::Vec::new)
                .ok_or(UnificationError::StringMismatch),

            (TermView::Variable(v), other) | (other, TermView::Variable(v)) => {
                Ok(vec![BindingConstraint::new(v.id, other)])
            }

            (TermView::List(l1), TermView::List(l2)) if l1.len() == l2.len() => {
                let mut constraints = Vec::new();
                for (a1, a2) in l1.iter().zip(l2.iter()) {
                    constraints.extend(a1.clone().collect_constraints(a2.clone())?);
                }
                Ok(constraints)
            }
            (TermView::List(_), TermView::List(_)) => Err(UnificationError::ArityMismatch),

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

            (Term::Variable(v), other) => v.collect_constraints(other),
            (other, TermView::Variable(v)) => Ok(vec![BindingConstraint::new(v.id, other)]),

            (
                Term::Literal(Literal { negated: n1, .. }),
                TermView::Literal(LiteralView { negated: n2, .. }),
            ) if *n1 != n2 => Err(UnificationError::NegationMismatch),
            (
                Term::Literal(Literal { structure: s1, .. }),
                TermView::Literal(LiteralView { structure: s2, .. }),
            ) => s1.collect_constraints(s2),

            (Term::Number(n1), TermView::Number(n2)) => (*n1 == n2)
                .then(alloc::vec::Vec::new)
                .ok_or(UnificationError::NumberMismatch),
            (Term::String(a), TermView::String(b)) => (a.as_bstr() == b.as_bstr())
                .then(alloc::vec::Vec::new)
                .ok_or(UnificationError::StringMismatch),

            (Term::List(l1), TermView::List(l2)) if l1.len() == l2.len() => {
                let mut constraints = Vec::new();
                for (a1, a2) in l1.iter().zip(l2.iter()) {
                    constraints.extend(a1.collect_constraints(a2.clone())?);
                }
                Ok(constraints)
            }
            (Term::List(_), TermView::List(_)) => Err(UnificationError::ArityMismatch),

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

        let args1 = self.arguments.as_deref().unwrap_or(&[]);
        let args2 = other.arguments.as_deref().unwrap_or(&[]);

        if args1.len() != args2.len() {
            return Err(UnificationError::ArityMismatch);
        }

        let mut constraints = Vec::new();
        for (a1, a2) in args1.iter().zip(args2.iter()) {
            constraints.extend(a1.collect_constraints(a2)?);
        }
        Ok(constraints)
    }
}

impl<'a> UnifyView<'a> for StructureView<'a> {
    fn collect_constraints(self, other: Self) -> Result<Vec<BindingConstraint<'a>>> {
        if self.functor != other.functor {
            return Err(UnificationError::FunctorMismatch);
        }

        let args1 = self.arguments.as_deref().unwrap_or(&[]);
        let args2 = other.arguments.as_deref().unwrap_or(&[]);

        if args1.len() != args2.len() {
            return Err(UnificationError::ArityMismatch);
        }

        let mut constraints = Vec::new();
        for (a1, a2) in args1.iter().zip(args2.iter()) {
            constraints.extend(a1.clone().collect_constraints(a2.clone())?);
        }
        Ok(constraints)
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
        if self.functor != *other.functor {
            return Err(UnificationError::FunctorMismatch);
        }

        let args1 = self.arguments.as_deref().unwrap_or(&[]);
        let args2 = other.arguments.as_deref().unwrap_or(&[]);

        if args1.len() != args2.len() {
            return Err(UnificationError::ArityMismatch);
        }

        let mut constraints = Vec::new();
        for (a1, a2) in args1.iter().zip(args2.iter()) {
            constraints.extend(a1.collect_constraints(a2.clone())?);
        }
        Ok(constraints)
    }
}

impl Unify<&Term> for Variable {
    fn collect_constraints<'a>(&'a self, other: &'a Term) -> Result<Vec<BindingConstraint<'a>>>
    where
        Term: 'a,
    {
        Ok(vec![BindingConstraint::new(self.id, other)])
    }
}

impl<'v> Unify<TermView<'v>> for Variable {
    fn collect_constraints<'a>(&'a self, other: TermView<'v>) -> Result<Vec<BindingConstraint<'a>>>
    where
        TermView<'v>: 'a,
    {
        Ok(vec![BindingConstraint::new(self.id, other.clone())])
    }
}

impl Unify<&Literal> for Literal {
    fn collect_constraints<'a>(&'a self, other: &'a Self) -> Result<Vec<BindingConstraint<'a>>>
    where
        Self: 'a,
    {
        TermView::from(self).collect_constraints(TermView::from(other))
    }
}

#[cfg(test)]
mod tests {
    use bstr::BStr;

    use crate::term::view::ViewString;

    use super::*;

    fn view_string(s: &str) -> TermView<'_> {
        TermView::String(ViewString::from(BStr::new(s)))
    }

    // Regression: neither `TermView`-vs-`TermView` nor `Term`-vs-`TermView` unification had a
    // String arm, so both fell through to the `_ => TypeMismatch` catch-all instead of comparing
    // the strings - `Term`-vs-`&Term` string unification was already covered
    // (`unification::tests::mismatch_constants`), which is why this went unnoticed.
    #[test]
    fn view_vs_view_strings_unify_when_equal_and_mismatch_otherwise() {
        assert!(view_string("n0").unify(view_string("n0"), None).is_ok());
        assert_eq!(
            view_string("n0").unify(view_string("n1"), None).unwrap_err(),
            UnificationError::StringMismatch
        );
    }

    #[test]
    fn term_vs_view_strings_unify_when_equal_and_mismatch_otherwise() {
        let equal = Term::String("n0".into());
        assert!(equal.unify(view_string("n0"), None).is_ok());

        let different = Term::String("n0".into());
        assert_eq!(
            different.unify(view_string("n1"), None).unwrap_err(),
            UnificationError::StringMismatch
        );
    }
}
