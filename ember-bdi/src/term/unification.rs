use alloc::vec;
use alloc::vec::Vec;

use crate::bindings::{Bindings, StructureView, TermView};
use crate::term::{NonGround, Structure, Term};
use crate::variable::{Variable, VariableId};

pub(crate) type Result<T> = core::result::Result<T, UnificationError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnificationError {
    NumberMismatch,
    StringMismatch,
    FunctorMismatch,
    ArityMismatch,
    TypeMismatch,
    NegationMismatch,
    CyclicReference,
}

impl core::fmt::Display for UnificationError {
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

impl core::error::Error for UnificationError {}

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
    use alloc::vec::Vec;

    use crate::bindings::{AliasMap, Bindings, StructureView, TermView};
    use crate::term::{NonGround, Term};
    use crate::variable::VariableId;

    use super::{BindingConstraint, Result, UnificationError, UnifyView};

    pub(super) struct ConstraintSolver<'a> {
        classes: EquivalenceClasses<'a>,
        queue: Vec<BindingConstraint<'a>>,
    }

    impl<'a> ConstraintSolver<'a> {
        pub(super) fn new(constraints: impl IntoIterator<Item = BindingConstraint<'a>>) -> Self {
            Self {
                classes: EquivalenceClasses::default(),
                queue: constraints.into_iter().collect(),
            }
        }

        pub(super) fn load_existing_bindings(&mut self, existing: &Bindings<'a>) -> Result<()> {
            if let Some(bindings) = &existing.bindings {
                for (&variable, term) in bindings.iter() {
                    if let Some(term) = term {
                        self.classes.register(
                            BindingConstraint {
                                variable,
                                value: term.clone(),
                            },
                            &mut self.queue,
                        )?;
                    }
                }
            }

            for &(var1, var2) in existing.aliases.iter() {
                self.classes.merge(var1, var2, &mut self.queue)?;
            }

            Ok(())
        }

        pub(super) fn solve(mut self) -> Result<Bindings<'a>> {
            self.process_constraints()?;
            self.finalize()
        }

        fn process_constraints(&mut self) -> Result<()> {
            while let Some(constraint) = self.queue.pop() {
                self.classes.register(constraint, &mut self.queue)?;
            }
            Ok(())
        }

        fn finalize(&mut self) -> Result<Bindings<'a>> {
            let variables: Vec<VariableId> = self.classes.parent.keys().copied().collect();
            for &var in &variables {
                self.classes.find_root(var);
            }

            let mut root_assignments = BTreeMap::new();
            for (&root, term) in &self.classes.root_to_term {
                let resolved = self.classes.resolve_term(term.clone(), &mut Vec::new())?;
                root_assignments.insert(root, resolved);
            }

            let mut bindings = BTreeMap::new();
            let mut root_to_vars: BTreeMap<VariableId, Vec<VariableId>> = BTreeMap::new();

            for &var in &variables {
                let root = self.classes.find_root(var);
                bindings.insert(var, root_assignments.get(&root).cloned());
                root_to_vars.entry(root).or_default().push(var);
            }

            let aliases = self.extract_aliases(root_to_vars);

            Ok(Bindings::new(bindings, aliases))
        }

        fn extract_aliases(&self, root_to_vars: BTreeMap<VariableId, Vec<VariableId>>) -> AliasMap {
            let mut aliases_pairs = Vec::new();
            for vars in root_to_vars.values() {
                if let Some((&first, rest)) = vars.split_first() {
                    for &other in rest {
                        aliases_pairs.push((first, other));
                    }
                }
            }
            AliasMap::new(aliases_pairs)
        }
    }

    #[derive(Debug, Default)]
    struct EquivalenceClasses<'a> {
        parent: BTreeMap<VariableId, VariableId>,
        root_to_term: BTreeMap<VariableId, TermView<'a>>,
    }

    impl<'a> EquivalenceClasses<'a> {
        fn find_root(&mut self, var: VariableId) -> VariableId {
            let p = *self.parent.entry(var).or_insert(var);
            if p != var {
                let root = self.find_root(p);
                self.parent.insert(var, root);
                root
            } else {
                p
            }
        }

        fn root_of(&self, var: VariableId) -> Option<VariableId> {
            let mut current = var;
            loop {
                match self.parent.get(&current) {
                    Some(&p) if p == current => return Some(current),
                    Some(&p) => current = p,
                    None => return None,
                }
            }
        }

        fn merge(
            &mut self,
            var1: VariableId,
            var2: VariableId,
            queue: &mut Vec<BindingConstraint<'a>>,
        ) -> Result<()> {
            let root1 = self.find_root(var1);
            let root2 = self.find_root(var2);

            if root1 != root2 {
                self.parent.insert(root2, root1);

                if let Some(t2) = self.root_to_term.remove(&root2) {
                    self.add_term(root1, t2, queue)?;
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
                let root = self.find_root(variable);
                self.add_term(root, value, queue)
            }
        }

        fn add_term(
            &mut self,
            root: VariableId,
            term: TermView<'a>,
            queue: &mut Vec<BindingConstraint<'a>>,
        ) -> Result<()> {
            if let Some(t1) = self.root_to_term.get(&root) {
                queue.extend(t1.clone().collect_constraints(term)?);
            } else {
                self.root_to_term.insert(root, term);
            }
            Ok(())
        }

        fn resolve_term(
            &self,
            term: TermView<'a>,
            visiting: &mut Vec<VariableId>,
        ) -> Result<TermView<'a>> {
            match term {
                TermView::Term(term) => match term {
                    Term::Number(n) => Ok(TermView::Number(*n)),
                    Term::String(_) => Ok(TermView::Term(term)),

                    Term::Variable(NonGround(v)) => {
                        let Some(root) = self.root_of(v.id) else {
                            return Ok(TermView::Term(term));
                        };
                        Ok(self
                            .resolve_root(root, visiting)?
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
                TermView::Number(_) => Ok(term.clone()),
            }
        }

        fn resolve_root(
            &self,
            root: VariableId,
            visiting: &mut Vec<VariableId>,
        ) -> Result<Option<TermView<'a>>> {
            if visiting.contains(&root) {
                return Err(UnificationError::CyclicReference);
            }
            let Some(term) = self.root_to_term.get(&root) else {
                return Ok(None);
            };

            visiting.push(root);
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
            solver.load_existing_bindings(existing_bindings)?;
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
            UnificationError::NumberMismatch
        );

        let (s1, s2) = (s("a"), s("b"));
        assert_eq!(
            s1.unify(&s2, None).unwrap_err(),
            UnificationError::StringMismatch
        );
    }

    #[test]
    fn type_mismatch() {
        let (t1, t2) = (n(1.0), s("1"));
        assert_eq!(
            t1.unify(&t2, None).unwrap_err(),
            UnificationError::TypeMismatch
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
            UnificationError::ArityMismatch
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
            UnificationError::FunctorMismatch
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
            UnificationError::NegationMismatch
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
            UnificationError::NumberMismatch
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
            UnificationError::CyclicReference
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
            UnificationError::CyclicReference
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
        assert_eq!(err, UnificationError::NumberMismatch);
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
