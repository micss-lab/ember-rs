use alloc::vec;
use alloc::vec::Vec;

use crate::bindings::Bindings;
use crate::term::{NonGround, Structure, Term};
use crate::variable::Variable;

pub(crate) type Result<T> = core::result::Result<T, UnificationFailedError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnificationFailedError {
    NumberMismatch,
    StringMismatch,
    FunctorMismatch,
    ArityMismatch,
    TypeMismatch,
    NegationMismatch,
    CyclicReference,
}

impl core::fmt::Display for UnificationFailedError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "unification failed: ")?;
        match self {
            Self::NumberMismatch => write!(f, "number mismatch"),
            Self::StringMismatch => write!(f, "string mismatch"),
            Self::FunctorMismatch => write!(f, "functor mismatch"),
            Self::ArityMismatch => write!(f, "arity mismatch"),
            Self::TypeMismatch => write!(f, "type mismatch"),
            Self::NegationMismatch => write!(f, "negation mismatch"),
            Self::CyclicReference => write!(f, "cyclic reference detected"),
        }
    }
}

impl core::error::Error for UnificationFailedError {}

pub trait Unify<Rhs = Self> {
    /// Collect individual constraints without recursive verification that they are collectively
    /// sound.
    fn collect_constraints<'a>(&'a self, other: &'a Rhs) -> Result<Vec<BindingConstraint<'a>>>;

    /// Try to unify this structure with something it can be unified with.
    fn unify<'a>(&'a self, other: &'a Rhs) -> Result<Bindings<'a>> {
        Bindings::build_from_constraints(self.collect_constraints(other)?)
    }
}

impl Unify for Term {
    fn collect_constraints<'a>(&'a self, other: &'a Term) -> Result<Vec<BindingConstraint<'a>>> {
        use Term::*;

        match (self, other) {
            (Variable(NonGround(v)), t) | (t, Variable(NonGround(v))) => v.collect_constraints(t),

            (Number(n1), Number(n2)) if n1 == n2 => Ok(vec![]),
            (Number(_), Number(_)) => Err(UnificationFailedError::NumberMismatch),

            (String(s1), String(s2)) if s1 == s2 => Ok(vec![]),
            (String(_), String(_)) => Err(UnificationFailedError::StringMismatch),

            (Literal { negated: n1, .. }, Literal { negated: n2, .. }) if n1 != n2 => {
                Err(UnificationFailedError::NegationMismatch)
            }
            (Literal { structure: s1, .. }, Literal { structure: s2, .. }) => {
                s1.collect_constraints(s2)
            }
            _ => Err(UnificationFailedError::TypeMismatch),
        }
    }
}

impl Unify for Structure {
    fn collect_constraints<'a>(
        &'a self,
        other: &'a Structure,
    ) -> Result<Vec<BindingConstraint<'a>>> {
        if self.functor != other.functor {
            return Err(UnificationFailedError::FunctorMismatch);
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
            _ => Err(UnificationFailedError::ArityMismatch),
        }
    }
}

impl Unify<Term> for Variable {
    fn collect_constraints<'a>(&'a self, other: &'a Term) -> Result<Vec<BindingConstraint<'a>>> {
        // TODO: Check that the term can be converted to the type of the variable.

        Ok(vec![BindingConstraint::new(self, other)])
    }
}

#[derive(Debug)]
pub struct BindingConstraint<'a> {
    variable: &'a Variable,
    value: &'a Term,
}

impl<'a> BindingConstraint<'a> {
    pub fn new(variable: &'a Variable, value: &'a Term) -> Self {
        Self { variable, value }
    }
}

mod solver {
    use alloc::borrow::Cow;
    use alloc::collections::BTreeMap;
    use alloc::vec::Vec;

    use crate::bindings::Bindings;
    use crate::term::{NonGround, Term};
    use crate::variable::{Variable, VariableId};

    use super::{BindingConstraint, Result, UnificationFailedError, Unify};

    pub(super) struct ConstraintSolver<'a> {
        partitions: Partitions<'a>,
        queue: Vec<BindingConstraint<'a>>,
    }

    impl<'a> ConstraintSolver<'a> {
        pub(super) fn new(constraints: impl IntoIterator<Item = BindingConstraint<'a>>) -> Self {
            Self {
                partitions: Partitions::default(),
                queue: constraints.into_iter().collect(),
            }
        }

        pub(super) fn solve(mut self) -> Result<Bindings<'a>> {
            while let Some(BindingConstraint { variable, value }) = self.queue.pop() {
                if let Term::Variable(NonGround(alias)) = value {
                    self.partitions.merge(variable, alias, &mut self.queue)?;
                } else {
                    let pid = self.partitions.get_or_create(variable);
                    self.partitions.add_term(pid, value, &mut self.queue)?;
                }
            }

            let mut partition_assignments = BTreeMap::new();
            for pid in self.partitions.variable_to_partition.values() {
                if partition_assignments.contains_key(pid) {
                    continue;
                }
                if let Some(term) = self.partitions.partition_to_term.get(pid) {
                    let term = self.partitions.resolve_term(term, &mut Vec::new())?;
                    partition_assignments.insert(*pid, term);
                }
            }

            let bindings = self
                .partitions
                .variable_to_partition
                .into_iter()
                .map(|(vid, pid)| (vid, partition_assignments.get(&pid).cloned()));
            Ok(Bindings::new(bindings))
        }
    }

    type PartitionId = usize;

    #[derive(Default)]
    struct Partitions<'a> {
        next_id: PartitionId,
        variable_to_partition: BTreeMap<VariableId, PartitionId>,
        partition_to_term: BTreeMap<PartitionId, &'a Term>,
    }

    impl<'a> Partitions<'a> {
        fn get_or_create(&mut self, variable: &Variable) -> PartitionId {
            *self
                .variable_to_partition
                .entry(variable.id)
                .or_insert_with(|| {
                    let id = self.next_id;
                    self.next_id += 1;
                    id
                })
        }

        fn merge(
            &mut self,
            variable: &'a Variable,
            alias: &'a Variable,
            queue: &mut Vec<BindingConstraint<'a>>,
        ) -> Result<()> {
            let pid1 = self.get_or_create(variable);
            let pid2 = self.get_or_create(alias);

            if pid1 != pid2 {
                // Update all mappings pointing to pid2 to point to pid1
                for pid in self.variable_to_partition.values_mut() {
                    if *pid == pid2 {
                        *pid = pid1;
                    }
                }

                if let Some(t2) = self.partition_to_term.remove(&pid2) {
                    self.add_term(pid1, t2, queue)?;
                }
            }

            Ok(())
        }

        fn add_term(
            &mut self,
            pid: PartitionId,
            term: &'a Term,
            queue: &mut Vec<BindingConstraint<'a>>,
        ) -> Result<()> {
            if let Some(t1) = self.partition_to_term.get(&pid) {
                queue.extend(t1.collect_constraints(term)?);
            } else {
                self.partition_to_term.insert(pid, term);
            }
            Ok(())
        }

        /// Try to resolve the term as far as possible. If a variable does not have a
        /// value, return it as is.
        fn resolve_term(
            &self,
            term: &'a Term,
            visiting: &mut Vec<PartitionId>,
        ) -> Result<Cow<'a, Term>> {
            match term {
                Term::Number(_) | Term::String(_) => Ok(Cow::Borrowed(term)),

                Term::Variable(NonGround(v)) => {
                    let Some(pid) = self.variable_to_partition.get(&v.id) else {
                        return Ok(Cow::Borrowed(term));
                    };
                    Ok(self
                        .resolve_pid(*pid, visiting)?
                        .unwrap_or_else(|| Cow::Borrowed(term)))
                }
                Term::Literal {
                    negated: n,
                    structure: s,
                } => {
                    let args = match &s.arguments {
                        Some(args) => {
                            let mut resolved_args = Vec::with_capacity(args.len());
                            for arg in args.iter() {
                                resolved_args.push(self.resolve_term(arg, visiting)?.into_owned());
                            }
                            Some(resolved_args)
                        }
                        None => None,
                    };
                    Ok(Cow::Owned(Term::Literal {
                        negated: *n,
                        structure: crate::term::Structure {
                            functor: s.functor.clone(),
                            arguments: args,
                        },
                    }))
                }
            }
        }

        fn resolve_pid(
            &self,
            pid: PartitionId,
            visiting: &mut Vec<PartitionId>,
        ) -> Result<Option<Cow<'a, Term>>> {
            if visiting.contains(&pid) {
                return Err(UnificationFailedError::CyclicReference); // Cycle detected
            }
            let Some(term) = self.partition_to_term.get(&pid) else {
                return Ok(None);
            };

            visiting.push(pid);
            let result = self.resolve_term(term, visiting);
            visiting.pop();
            result.map(Some)
        }
    }
}

impl<'a> Bindings<'a> {
    /// Tries to build a unification map of the collected constraints.
    ///
    /// # Implementation
    ///
    /// The function does the following: given a collection of constraints, find or create the
    /// partition this variable belongs to. If the partition already contains a value, try to
    /// unify the current value with the new one returning new constraints. Do this for each
    /// constraint in the queue.
    fn build_from_constraints(
        constraints: impl IntoIterator<Item = BindingConstraint<'a>>,
    ) -> Result<Self> {
        solver::ConstraintSolver::new(constraints).solve()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::term::{Atom, NonGround, Structure, Term};
    use crate::variable::Variable;
    use alloc::borrow::Cow;

    fn n(number: f32) -> Term {
        Term::Number(number.into())
    }

    fn s(string: impl AsRef<str>) -> Term {
        Term::String(string.as_ref().into())
    }

    fn v() -> Variable {
        Variable::new()
    }

    fn tv(variable: &Variable) -> Term {
        Term::Variable(NonGround(variable.clone()))
    }

    fn structure(functor: &str, args: Vec<Term>) -> Structure {
        Structure {
            functor: Atom(functor.into()),
            arguments: Some(args),
        }
    }

    fn literal(negated: bool, functor: &str, args: Vec<Term>) -> Term {
        Term::Literal {
            negated,
            structure: structure(functor, args),
        }
    }

    // --- Happy Day Scenarios ---

    #[test]
    fn unify_identical_constants() {
        let (t1, t2) = (n(42.0), n(42.0));
        assert!(t1.unify(&t2).is_ok());

        let (s1, s2) = (s("hello"), s("hello"));
        assert!(s1.unify(&s2).is_ok());
    }

    #[test]
    fn simple_variable_binding() {
        let (x, val) = (v(), n(100.0));

        let result = x.unify(&val).expect("Unification failed");
        let binding = result.get(&x).expect("Variable 1 should be bound");
        assert_eq!(binding, &n(100.0));
    }

    #[test]
    fn structural_unification() {
        let x = v();

        // f(X, 2) == f(1, 2)
        let t1 = literal(false, "f", vec![tv(&x), n(2.0)]);
        let t2 = literal(false, "f", vec![n(1.0), n(2.0)]);

        let result = t1.unify(&t2).expect("Unification failed");
        assert_eq!(result.get(&x), Some(&n(1.0)));
    }

    #[test]
    fn variable_aliasing() {
        let (x, y) = (v(), v());

        // X == Y, Y == 42 => X == 42
        // We simulate this by unifying a structure that forces these constraints
        // pair(X, Y) == pair(Y, 42)
        let t1 = literal(false, "pair", vec![tv(&x), tv(&y)]);
        let t2 = literal(false, "pair", vec![tv(&y), n(42.0)]);

        let result = t1.unify(&t2).expect("Unification failed");
        assert_eq!(result.get(&x), Some(&n(42.0)));
        assert_eq!(result.get(&y), Some(&n(42.0)));
    }

    // --- Edge Cases & Failures ---

    #[test]
    fn mismatch_constants() {
        let (t1, t2) = (n(1.0), n(2.0));
        assert_eq!(
            t1.unify(&t2).unwrap_err(),
            UnificationFailedError::NumberMismatch
        );

        let (s1, s2) = (s("a"), s("b"));
        assert_eq!(
            s1.unify(&s2).unwrap_err(),
            UnificationFailedError::StringMismatch
        );
    }

    #[test]
    fn type_mismatch() {
        let (t1, t2) = (n(1.0), s("1"));
        assert_eq!(
            t1.unify(&t2).unwrap_err(),
            UnificationFailedError::TypeMismatch
        );
    }

    #[test]
    fn arity_mismatch() {
        let (t1, t2) = (
            literal(false, "f", vec![n(1.0)]),
            literal(false, "f", vec![n(1.0), n(2.0)]),
        );
        assert_eq!(
            t1.unify(&t2).unwrap_err(),
            UnificationFailedError::ArityMismatch
        );
    }

    #[test]
    fn functor_mismatch() {
        let (t1, t2) = (
            literal(false, "f", vec![n(1.0)]),
            literal(false, "g", vec![n(1.0)]),
        );
        assert_eq!(
            t1.unify(&t2).unwrap_err(),
            UnificationFailedError::FunctorMismatch
        );
    }

    #[test]
    fn negation_mismatch() {
        let (t1, t2) = (
            literal(true, "f", vec![n(1.0)]),
            literal(false, "f", vec![n(1.0)]),
        );
        assert_eq!(
            t1.unify(&t2).unwrap_err(),
            UnificationFailedError::NegationMismatch
        );
    }

    #[test]
    fn inconsistent_variable_binding() {
        let x = v();

        // pair(X, X) == pair(1, 2) -> Should fail because X cannot be 1 and 2
        let (t1, t2) = (
            literal(false, "pair", vec![tv(&x), tv(&x)]),
            literal(false, "pair", vec![n(1.0), n(2.0)]),
        );

        assert_eq!(
            t1.unify(&t2).unwrap_err(),
            UnificationFailedError::NumberMismatch
        );
    }

    // --- Complex Dependencies & Cycles ---

    #[test]
    fn recursive_resolution() {
        let (x, y) = (v(), v());

        // f(X) == f(g(Y)), Y == 1 => X should resolve to g(1)
        let (t1, t2) = (
            literal(false, "f", vec![tv(&x)]),
            literal(false, "f", vec![literal(false, "g", vec![tv(&y)])]),
        );

        // We need to provide the second constraint Y=1
        // We'll bundle them in a single unification: triple(f(X), Y) == triple(f(g(Y)), 1)
        let query = literal(false, "triple", vec![t1, tv(&y)]);
        let belief = literal(false, "triple", vec![t2, n(1.0)]);

        let result = query.unify(&belief).expect("Complex resolution failed");

        let x_binding = result.get(&x).expect("X should be bound");
        let expected_x = literal(false, "g", vec![n(1.0)]);
        assert_eq!(x_binding, &expected_x);
    }

    #[test]
    fn direct_cycle_detection() {
        let x = v();

        // X == f(X)
        let fx = literal(false, "f", vec![tv(&x)]);

        // Unification might succeed in step 1 (Walk), but should fail in solve (Resolution)
        assert_eq!(
            x.unify(&fx).unwrap_err(),
            UnificationFailedError::CyclicReference
        );
    }

    #[test]
    fn indirect_cycle_detection() {
        let (x, y) = (v(), v());

        // X == f(Y), Y == f(X)
        let t1 = literal(false, "pair", vec![tv(&x), tv(&y)]);
        let t2 = literal(
            false,
            "pair",
            vec![
                literal(false, "f", vec![tv(&y)]),
                literal(false, "f", vec![tv(&x)]),
            ],
        );

        assert_eq!(
            t1.unify(&t2).unwrap_err(),
            UnificationFailedError::CyclicReference
        );
    }

    #[test]
    fn deep_alias_chain_resolution() {
        let (a, b, c, d) = (v(), v(), v(), v());

        // A=B, B=C, C=D, D=E, E=100
        let t1 = literal(false, "chain", vec![tv(&a), tv(&b), tv(&c), tv(&d)]);
        let t2 = literal(false, "chain", vec![tv(&b), tv(&c), tv(&d), n(100.0)]);

        let result = t1.unify(&t2).expect("Deep chain failed");
        for v in [a, b, c, d] {
            assert_eq!(result.get(&v), Some(&n(100.0)));
        }
    }

    #[test]
    fn unbound_variables_in_result() {
        let (x, y, z) = (v(), v(), v());

        // f(X, Y) == f(1, Z)
        // X=1, Y=Z (Z is free)
        let t1 = literal(false, "f", vec![tv(&x), tv(&y)]);
        let t2 = literal(false, "f", vec![n(1.0), tv(&z)]);

        let result = t1.unify(&t2).expect("Unification with free vars failed");

        assert_eq!(result.get(&x), Some(&n(1.0)));

        assert_eq!(result.get(&y), None);
        assert_eq!(result.get(&z), None);
    }
}
