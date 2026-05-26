pub(crate) use self::error::Result;

pub mod constraint;
pub mod error;
pub mod traits;

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;
    use alloc::vec;
    use alloc::vec::Vec;

    use crate::bindings::{StructureView, TermView};
    use crate::term::{Atom, Structure, Term};
    use crate::unification::error::UnificationError;
    use crate::unification::traits::Unify;

    use crate::testing::*;

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
        let t1 = literal(false, "f", vec![vt(&x), n(2.0)]);
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
        let t1 = literal(false, "pair", vec![vt(&x), vt(&y)]);
        let t2 = literal(false, "pair", vec![vt(&y), n(42.0)]);

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
            literal(false, "pair", vec![vt(&x), vt(&x)]),
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
            literal(false, "f", vec![vt(&x)]),
            literal(false, "f", vec![literal(false, "g", vec![vt(&y)])]),
        );

        let query = literal(false, "triple", vec![t1, vt(&y)]);
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
        let fx = literal(false, "f", vec![vt(&x)]);

        assert_eq!(
            x.unify(&fx, None).unwrap_err(),
            UnificationError::CyclicReference
        );
    }

    #[test]
    fn indirect_cycle_detection() {
        let (x, y) = (v(), v());

        // X == f(Y), Y == f(X)
        let t1 = literal(false, "pair", vec![vt(&x), vt(&y)]);
        let t2 = literal(
            false,
            "pair",
            vec![
                literal(false, "f", vec![vt(&y)]),
                literal(false, "f", vec![vt(&x)]),
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
        let t1 = literal(false, "chain", vec![vt(&a), vt(&b), vt(&c), vt(&d)]);
        let t2 = literal(false, "chain", vec![vt(&b), vt(&c), vt(&d), n(100.0)]);

        let result = t1.unify(&t2, None).expect("Deep chain failed");
        for v in [a, b, c, d] {
            assert_eq!(result.get(&v), Some(&n(100.0).as_view()));
        }
    }

    #[test]
    fn unbound_variables_in_result() {
        let (x, y, z) = (v(), v(), v());

        let t1 = literal(false, "f", vec![vt(&x), vt(&y)]);
        let t2 = literal(false, "f", vec![n(1.0), vt(&z)]);

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
        let t1 = literal(false, "f", vec![vt(&x)]);
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
        let t1 = literal(false, "f", vec![vt(&x)]);
        let t2 = literal(false, "f", vec![n(2.0)]);

        let err = t1.unify(&t2, Some(&existing)).unwrap_err();
        assert_eq!(err, UnificationError::NumberMismatch);
    }

    // --- Existing bindings ---

    #[test]
    fn existing_alias_propagation() {
        let (x, y) = (v(), v());
        // Existing: X == Y
        let t_init1 = literal(false, "pair", vec![vt(&x), vt(&y)]);
        let t_init2 = literal(false, "pair", vec![vt(&y), vt(&x)]);
        let existing = t_init1.unify(&t_init2, None).unwrap();

        let t1 = vt(&x);
        let t2 = n(10.0);

        let result = t1.unify(&t2, Some(&existing)).expect("Aliasing failed");
        assert_eq!(result.get(&x), Some(&n(10.0).as_view()));
        assert_eq!(result.get(&y), Some(&n(10.0).as_view()));
    }

    #[test]
    fn existing_binding_deep_resolution() {
        let (x, y) = (v(), v());

        let yt = vt(&y);

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

        let t1 = literal(false, "f", vec![vt(&y)]);
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
