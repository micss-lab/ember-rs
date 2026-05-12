use alloc::vec;
use alloc::vec::Vec;

use crate::bindings::{Bindings, StructureView, TermView};
use crate::term::{NonGround, Structure, Term};
use crate::variable::{Variable, VariableId};

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

impl<'a> UnifyView<'a> for TermView<'a> {
    fn collect_constraints(self, other: Self) -> Result<Vec<BindingConstraint<'a>>> {
        match (self, other) {
            (TermView::Term(this), TermView::Term(other)) => this.collect_constraints(other),

            (TermView::Term(Term::Variable(NonGround(v))), t)
            | (t, TermView::Term(Term::Variable(NonGround(v)))) => v.collect_constraints(t),

            (TermView::Term(this), TermView::Literal { negated, structure })
            | (TermView::Literal { negated, structure }, TermView::Term(this)) => {
                match (this, (negated, structure)) {
                    (Term::Literal { negated: n1, .. }, (n2, _)) if *n1 != n2 => {
                        Err(UnificationFailedError::NegationMismatch)
                    }
                    (Term::Literal { structure: s1, .. }, (_, s2)) => s1.collect_constraints(s2),

                    _ => Err(UnificationFailedError::TypeMismatch),
                }
            }

            (TermView::Literal { negated: n1, .. }, TermView::Literal { negated: n2, .. })
                if n1 != n2 =>
            {
                Err(UnificationFailedError::NegationMismatch)
            }
            (TermView::Literal { structure: s1, .. }, TermView::Literal { structure: s2, .. }) => {
                s1.collect_constraints(s2)
            }
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

            (Term::Literal { negated: n1, .. }, TermView::Literal { negated: n2, .. })
                if *n1 != n2 =>
            {
                Err(UnificationFailedError::NegationMismatch)
            }
            (Term::Literal { structure: s1, .. }, TermView::Literal { structure: s2, .. }) => {
                s1.collect_constraints(s2)
            }

            _ => Err(UnificationFailedError::TypeMismatch),
        }
    }
}

impl Unify<&Structure> for Structure {
    fn collect_constraints<'a>(&'a self, other: &'a Self) -> Result<Vec<BindingConstraint<'a>>>
    where
        Self: 'a,
    {
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

impl<'a> UnifyView<'a> for StructureView<'a> {
    fn collect_constraints(self, other: Self) -> Result<Vec<BindingConstraint<'a>>> {
        if self.functor != other.functor {
            return Err(UnificationFailedError::FunctorMismatch);
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
            _ => Err(UnificationFailedError::ArityMismatch),
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
            return Err(UnificationFailedError::FunctorMismatch);
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
            _ => Err(UnificationFailedError::ArityMismatch),
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

#[derive(Debug)]
pub struct BindingConstraint<'a> {
    variable: VariableId,
    value: TermView<'a>,
}

impl<'a> BindingConstraint<'a> {
    pub fn new(variable: VariableId, value: impl Into<TermView<'a>>) -> Self {
        Self {
            variable,
            value: value.into(),
        }
    }
}

mod solver {
    use alloc::collections::BTreeMap;
    use alloc::collections::btree_set::BTreeSet;
    use alloc::vec::Vec;

    use crate::bindings::{Bindings, StructureView, TermView};
    use crate::term::{NonGround, Term};
    use crate::variable::{Variable, VariableId};

    use super::{BindingConstraint, Result, UnificationFailedError, Unify, UnifyView};

    pub(super) struct ConstraintSolver<'a, 'b> {
        partitions: Partitions<'a>,
        queue: Vec<BindingConstraint<'a>>,
        existing_bindings: Option<&'b Bindings<'a>>,
    }

    impl<'a, 'b> ConstraintSolver<'a, 'b> {
        pub(super) fn new(constraints: impl IntoIterator<Item = BindingConstraint<'a>>) -> Self {
            Self {
                partitions: Partitions::default(),
                queue: constraints.into_iter().collect(),
                existing_bindings: None,
            }
        }

        pub(super) fn with_existing_bindings(
            mut self,
            existing_bindings: &'b Bindings<'a>,
        ) -> Self {
            self.existing_bindings = Some(existing_bindings);
            self
        }
    }

    impl<'a> ConstraintSolver<'a, '_> {
        pub(super) fn solve(mut self) -> Result<Bindings<'a>> {
            if let Some(existing_bindings) = self.existing_bindings {
                for (&variable, term) in existing_bindings.bindings.iter() {
                    let Some(term) = term else {
                        continue;
                    };
                    self.partitions.register(
                        BindingConstraint {
                            variable,
                            value: term.clone(),
                        },
                        &mut self.queue,
                    )?;
                }

                for (variable, aliases) in existing_bindings.aliases.iter() {
                    for alias in aliases {
                        self.partitions.merge(*variable, *alias, &mut self.queue)?;
                    }
                }
            }

            while let Some(constraint) = self.queue.pop() {
                self.partitions.register(constraint, &mut self.queue)?;
            }

            let bindings = {
                // 1. Resolve each partition exactly once
                let mut partition_assignments = BTreeMap::new();
                for (&pid, term) in &self.partitions.partition_to_term {
                    let resolved = self
                        .partitions
                        .resolve_term(term.clone(), &mut Vec::new())?;
                    partition_assignments.insert(pid, resolved);
                }

                // 2. Map variables to their resolved partition values
                let mut bindings = self
                    .partitions
                    .variable_to_partition
                    .iter()
                    .map(|(&vid, &pid)| (vid, partition_assignments.get(&pid).cloned()))
                    .collect::<BTreeMap<_, _>>();

                let aliases = {
                    let mut aliases: BTreeMap<PartitionId, BTreeSet<VariableId>> = BTreeMap::new();
                    for (variable, pid) in self.partitions.variable_to_partition.iter() {
                        aliases.entry(*pid).or_default().insert(*variable);
                    }
                    aliases.into_iter().map(|(_, v)| v).collect::<Vec<_>>()
                };

                Bindings::new(bindings, aliases)
            };

            Ok(bindings)
        }
    }

    type PartitionId = usize;

    #[derive(Debug, Default)]
    struct Partitions<'a> {
        next_id: PartitionId,
        variable_to_partition: BTreeMap<VariableId, PartitionId>,
        partition_to_term: BTreeMap<PartitionId, TermView<'a>>,
    }

    impl<'a> Partitions<'a> {
        fn get_or_create(&mut self, variable: VariableId) -> PartitionId {
            *self
                .variable_to_partition
                .entry(variable)
                .or_insert_with(|| {
                    let id = self.next_id;
                    self.next_id += 1;
                    id
                })
        }

        fn merge(
            &mut self,
            variable: VariableId,
            alias: VariableId,
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

        fn register(
            &mut self,
            BindingConstraint { variable, value }: BindingConstraint<'a>,
            queue: &mut Vec<BindingConstraint<'a>>,
        ) -> Result<()> {
            if let Some(alias) = value.as_variable() {
                self.merge(variable, alias.id, queue)
            } else {
                let pid = self.get_or_create(variable);
                self.add_term(pid, value, queue)
            }
        }

        fn add_term(
            &mut self,
            pid: PartitionId,
            term: TermView<'a>,
            queue: &mut Vec<BindingConstraint<'a>>,
        ) -> Result<()> {
            if let Some(t1) = self.partition_to_term.get(&pid) {
                queue.extend(t1.clone().collect_constraints(term)?);
            } else {
                self.partition_to_term.insert(pid, term);
            }
            Ok(())
        }

        /// Try to resolve the term as far as possible. If a variable does not have a
        /// value, return it as is.
        fn resolve_term(
            &self,
            term: TermView<'a>,
            visiting: &mut Vec<PartitionId>,
        ) -> Result<TermView<'a>> {
            match term {
                TermView::Term(term) => match term {
                    Term::Number(_) | Term::String(_) => Ok(TermView::Term(term)),

                    Term::Variable(NonGround(v)) => {
                        let Some(pid) = self.variable_to_partition.get(&v.id) else {
                            return Ok(TermView::Term(term));
                        };
                        Ok(self
                            .resolve_pid(*pid, visiting)?
                            .unwrap_or(TermView::Term(term)))
                    }
                    Term::Literal {
                        negated: n,
                        structure: s,
                    } => {
                        let args = match &s.arguments {
                            Some(args) => {
                                let mut resolved_args = Vec::with_capacity(args.len());
                                for arg in args.iter() {
                                    resolved_args
                                        .push(self.resolve_term(TermView::Term(arg), visiting)?);
                                }
                                Some(resolved_args.into_boxed_slice())
                            }
                            None => None,
                        };
                        Ok(TermView::Literal {
                            negated: *n,
                            structure: StructureView {
                                functor: &s.functor,
                                arguments: args,
                            },
                        })
                    }
                },
                TermView::Literal { negated, structure } => {
                    let args = match structure.arguments {
                        Some(args) => {
                            let mut resolved_args = Vec::with_capacity(args.len());
                            for arg in args {
                                resolved_args.push(self.resolve_term(arg, visiting)?);
                            }
                            Some(resolved_args.into_boxed_slice())
                        }
                        None => None,
                    };
                    Ok(TermView::Literal {
                        negated,
                        structure: StructureView {
                            functor: structure.functor,
                            arguments: args,
                        },
                    })
                }
            }
        }

        fn resolve_pid(
            &self,
            pid: PartitionId,
            visiting: &mut Vec<PartitionId>,
        ) -> Result<Option<TermView<'a>>> {
            if visiting.contains(&pid) {
                return Err(UnificationFailedError::CyclicReference); // Cycle detected
            }
            let Some(term) = self.partition_to_term.get(&pid) else {
                return Ok(None);
            };

            visiting.push(pid);
            let result = self.resolve_term(term.clone(), visiting);
            visiting.pop();
            result.map(Some)
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
    fn build_from_constraints<'b>(
        constraints: impl IntoIterator<Item = BindingConstraint<'a>>,
        existing_bindings: Option<&Bindings<'a>>,
    ) -> Result<Self> {
        let mut solver = solver::ConstraintSolver::new(constraints);
        if let Some(existing_bindings) = existing_bindings {
            solver = solver.with_existing_bindings(existing_bindings);
        }
        solver.solve()
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use super::*;
    use crate::term::{Atom, NonGround, Structure, Term};
    use crate::variable::Variable;

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

    fn structure(functor: &str, args: impl Into<Vec<Term>>) -> Structure {
        Structure {
            functor: Atom(functor.into()),
            arguments: Some(args.into().into_boxed_slice()),
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
        assert!(t1.unify(&t2, None).is_ok());

        let (s1, s2) = (s("hello"), s("hello"));
        assert!(s1.unify(&s2, None).is_ok());
    }

    #[test]
    fn simple_variable_binding() {
        let (x, val) = (v(), n(100.0));

        let result = x.unify(&val, None).expect("Unification failed");
        let binding = result.get(&x).expect("Variable 1 should be bound");
        assert_eq!(binding, &n(100.0).as_view());
    }

    #[test]
    fn structural_unification() {
        let x = v();

        // f(X, 2) == f(1, 2)
        let t1 = literal(false, "f", vec![tv(&x), n(2.0)]);
        let t2 = literal(false, "f", vec![n(1.0), n(2.0)]);

        let result = t1.unify(&t2, None).expect("Unification failed");
        assert_eq!(result.get(&x), Some(n(1.0).as_view()).as_ref());
    }

    #[test]
    fn variable_aliasing() {
        let (x, y) = (v(), v());

        // X == Y, Y == 42 => X == 42
        // We simulate this by unifying a structure that forces these constraints
        // pair(X, Y) == pair(Y, 42)
        let t1 = literal(false, "pair", vec![tv(&x), tv(&y)]);
        let t2 = literal(false, "pair", vec![tv(&y), n(42.0)]);

        let result = t1.unify(&t2, None).expect("Unification failed");
        assert_eq!(result.get(&x), Some(&n(42.0).as_view()));
        assert_eq!(result.get(&y), Some(&n(42.0).as_view()));
    }

    // --- Edge Cases & Failures ---

    #[test]
    fn mismatch_constants() {
        let (t1, t2) = (n(1.0), n(2.0));
        assert_eq!(
            t1.unify(&t2, None).unwrap_err(),
            UnificationFailedError::NumberMismatch
        );

        let (s1, s2) = (s("a"), s("b"));
        assert_eq!(
            s1.unify(&s2, None).unwrap_err(),
            UnificationFailedError::StringMismatch
        );
    }

    #[test]
    fn type_mismatch() {
        let (t1, t2) = (n(1.0), s("1"));
        assert_eq!(
            t1.unify(&t2, None).unwrap_err(),
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
            t1.unify(&t2, None).unwrap_err(),
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
            t1.unify(&t2, None).unwrap_err(),
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
            t1.unify(&t2, None).unwrap_err(),
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
            t1.unify(&t2, None).unwrap_err(),
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

        let query = literal(false, "triple", vec![t1, tv(&y)]);
        let belief = literal(false, "triple", vec![t2, n(1.0)]);

        let result = query
            .unify(&belief, None)
            .expect("Complex resolution failed");

        let x_binding = result.get(&x).expect("X should be bound");
        let (g, n) = (Atom("g".into()), n(1.0));
        let expected_x = TermView::Literal {
            negated: false,
            structure: StructureView {
                functor: &g,
                arguments: Some(Box::new([n.as_view()])),
            },
        };
        assert_eq!(x_binding, &expected_x);
    }

    #[test]
    fn direct_cycle_detection() {
        let x = v();

        // X == f(X)
        let fx = literal(false, "f", vec![tv(&x)]);

        assert_eq!(
            x.unify(&fx, None).unwrap_err(),
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
            t1.unify(&t2, None).unwrap_err(),
            UnificationFailedError::CyclicReference
        );
    }

    #[test]
    fn deep_alias_chain_resolution() {
        let (a, b, c, d) = (v(), v(), v(), v());

        // A=B, B=C, C=D, D=E, E=100
        let t1 = literal(false, "chain", vec![tv(&a), tv(&b), tv(&c), tv(&d)]);
        let t2 = literal(false, "chain", vec![tv(&b), tv(&c), tv(&d), n(100.0)]);

        let result = t1.unify(&t2, None).expect("Deep chain failed");
        for v in [a, b, c, d] {
            assert_eq!(result.get(&v), Some(&n(100.0).as_view()));
        }
    }

    #[test]
    fn unbound_variables_in_result() {
        let (x, y, z) = (v(), v(), v());

        let t1 = literal(false, "f", vec![tv(&x), tv(&y)]);
        let t2 = literal(false, "f", vec![n(1.0), tv(&z)]);

        let result = t1
            .unify(&t2, None)
            .expect("Unification with free vars failed");

        assert_eq!(result.get(&x), Some(&n(1.0).as_view()));

        assert_eq!(result.get(&y), None);
        assert_eq!(result.get(&z), None);
    }

    #[test]
    fn unify_with_existing_compatible_binding() {
        let x = v();
        let n1 = n(1.0);
        let existing = x.unify(&n1, None).unwrap();

        // f(X) == f(1) where X is already 1
        let t1 = literal(false, "f", vec![tv(&x)]);
        let t2 = literal(false, "f", vec![n(1.0)]);

        let result = t1.unify(&t2, Some(&existing)).expect("Should succeed");
        assert_eq!(result.get(&x), Some(&n(1.0).as_view()));
    }

    #[test]
    fn unify_with_existing_incompatible_binding() {
        let x = v();
        let n1 = n(1.0);
        let existing = x.unify(&n1, None).unwrap();

        // f(X) == f(2) where X is already 1 -> Should fail
        let t1 = literal(false, "f", vec![tv(&x)]);
        let t2 = literal(false, "f", vec![n(2.0)]);

        let err = t1.unify(&t2, Some(&existing)).unwrap_err();
        assert_eq!(err, UnificationFailedError::NumberMismatch);
    }

    // --- Existing bindings ---

    #[test]
    fn existing_alias_propagation() {
        let (x, y) = (v(), v());
        // Existing: X == Y
        let t_init1 = literal(false, "pair", vec![tv(&x), tv(&y)]);
        let t_init2 = literal(false, "pair", vec![tv(&y), tv(&x)]);
        let existing = t_init1.unify(&t_init2, None).unwrap();

        let t1 = tv(&x);
        let t2 = n(10.0);

        let result = t1.unify(&t2, Some(&existing)).expect("Aliasing failed");
        assert_eq!(result.get(&x), Some(&n(10.0).as_view()));
        assert_eq!(result.get(&y), Some(&n(10.0).as_view()));
    }

    #[test]
    fn existing_binding_deep_resolution() {
        let (x, y) = (v(), v());

        let yt = tv(&y);

        let term_g_y = TermView::Literal {
            negated: false,
            structure: StructureView {
                functor: &Atom("g".into()),
                arguments: Some(Box::new([TermView::Term(&yt)])),
            },
        };
        let existing = x
            .unify(term_g_y.clone(), None)
            .expect("Initial binding failed");

        assert_eq!(existing.get(&x), Some(term_g_y).as_ref());

        let t1 = literal(false, "f", vec![tv(&y)]);
        let t2 = literal(false, "f", vec![n(10.0)]);

        let final_bindings = t1
            .unify(&t2, Some(&existing))
            .expect("Deep resolution unification failed");

        let n = n(10.0);

        let expected_x = TermView::Literal {
            negated: false,
            structure: StructureView {
                functor: &Atom("g".into()),
                arguments: Some(alloc::boxed::Box::new([n.as_view()])),
            },
        };

        let x_res = final_bindings.get(&x).expect("X should still be bound");
        assert_eq!(x_res, &expected_x);

        assert_eq!(final_bindings.get(&y), Some(&n.as_view()));
    }
}
