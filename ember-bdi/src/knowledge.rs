use alloc::collections::BTreeMap;

use crate::bindings::Bindings;
use crate::literal::Literal;
use crate::term::unification::{UnificationFailedError, Unify};
use crate::term::{Atom, Ground, NonGround, Structure, Term, unification};

use self::query::Query;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Belief(
    /// While a belief is always a ground literal, storing one is done as a non-ground term.
    /// This is done to allow borrowing internal terms without first converting to a non-ground
    /// literal for unification.
    ///
    /// NOTE: To avoid this, we could use a wrapper type `LiteralTerm` which is the return
    /// value of the unification process for a variable, however, this would make the api more
    /// complicated for the user converting this dynamic type to their own parsed
    /// structure. This might still be done in the future.
    Term<NonGround>,
);

impl Belief {
    fn from_ground_literal(literal: Literal<Ground>) -> Self {
        Self(literal.into_non_ground().into())
    }

    /// Normalize the belief into a version that is more efficient when looking up both the
    /// positive and negative variants.
    fn normalize(self) -> (NormalizedBelief, BeliefMetadata) {
        let mut metadata = BeliefMetadata::default();
        let (negated, structure) = self.into_literal_atom_non_ground();
        metadata.negated = negated;
        (
            NormalizedBelief(Belief(Term::Literal {
                negated: false,
                structure,
            })),
            metadata,
        )
    }

    /// Return the internal Literal [`Literal::Atom`] data. It is guaranteed that this exists as
    /// this is an invariant of the type. The return type can unfortunately not be a ground
    /// structure as it is not stored as such.
    fn into_literal_atom_non_ground(self) -> (bool, Structure<NonGround>) {
        match self.0 {
            Term::Literal { negated, structure } => (negated, structure),
            Term::Number(_) | Term::String(_) | Term::Variable(_) => {
                unreachable!("belief is always a ground literal")
            }
        }
    }

    /// Return a reference to the internal [`Literal::Atom`] data. It is guaranteed that this exists as
    /// this is an invariant of the type. The return type can unfortunately not be a ground
    /// structure as it is not stored as such.
    fn as_literal_atom(&self) -> (&bool, &Structure<NonGround>) {
        match &self.0 {
            Term::Literal { negated, structure } => (negated, structure),
            Term::Number(_) | Term::String(_) | Term::Variable(_) => {
                unreachable!("belief is always a ground literal")
            }
        }
    }
}

impl Belief {
    fn atom_and_arity(&self) -> (Atom, usize) {
        let (_, structure) = self.as_literal_atom();
        atom_and_arity(structure)
    }
}

#[derive(Default)]
pub struct BeliefBase {
    /// Mapping from the belief atom and the arity to a list of ground truths.
    beliefs: BTreeMap<(Atom, usize), BeliefCollection>,
}

impl BeliefBase {
    /// Adds the belief to the belief-base. Returns `true` if the belief was already present.
    pub fn assert(&mut self, belief: impl Into<Belief>) -> bool {
        let belief = belief.into();
        let beliefs = self.beliefs.entry(belief.atom_and_arity()).or_default();
        beliefs.store(belief)
    }

    /// Removes the belief from the belief-base. Returns `true` if the belief has been removed.
    pub fn remove(&mut self, belief: impl Into<Belief>) -> bool {
        let belief = belief.into();
        let Some(beliefs) = self.beliefs.get_mut(&belief.atom_and_arity()) else {
            return false;
        };
        beliefs.remove(belief)
    }

    pub fn query<'a>(&'a self, query: &'a Literal) -> Query<'a> {
        let beliefs = match query {
            Literal::Atom { structure, .. } => self
                .beliefs
                .get(&atom_and_arity(structure))
                .map(|b| b.0.iter()),
            Literal::Variable(NonGround(_)) => {
                unimplemented!("a single variable as a query is not supported")
            }
        };
        Query::new(beliefs, query)
    }
}

/// A collection of normalized beliefs. Beliefs are always stored normalized with additional
/// metadata on how to construct their original version.
#[derive(Debug, Default)]
struct BeliefCollection(BTreeMap<NormalizedBelief, BeliefMetadata>);

impl BeliefCollection {
    /// Stores the belief in the collection returning `true` if the belief was new.
    fn store(&mut self, belief: Belief) -> bool {
        use alloc::collections::btree_map::Entry;

        let (belief, metadata) = belief.normalize();
        match self.0.entry(belief) {
            Entry::Vacant(entry) => {
                entry.insert(metadata);
                true
            }
            Entry::Occupied(mut entry) => entry.get_mut().update(metadata),
        }
    }

    /// Removes the belief from the collection if it was previously stored.
    fn remove(&mut self, belief: Belief) -> bool {
        use alloc::collections::btree_map::Entry;

        let (belief, metadata) = belief.normalize();
        match self.0.entry(belief) {
            Entry::Vacant(_) => false,
            Entry::Occupied(mut entry) => {
                let (updated, should_remove) = entry.get_mut().remove(metadata);
                if should_remove {
                    entry.remove();
                }
                updated || should_remove
            }
        }
    }
}

/// A normalized belief is a structure storing an easy look-up version of a belief. For example, a
/// top-level literal is always positive. This allows O(logn) lookup in BTrees even when the belief
/// is negated. To reconstruct the original belief, a [`BeliefMetadata`] instance is required.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct NormalizedBelief(Belief);

impl NormalizedBelief {
    /// Construct a belief from the normalized version and the accompanying metadata.
    fn denormalize(self, metadata: BeliefMetadata) -> Belief {
        use core::ops::BitXor;

        let (negated, structure) = self.0.into_literal_atom_non_ground();

        Belief(Term::Literal {
            negated: negated.bitxor(&metadata.negated),
            structure,
        })
    }

    fn unify_literal<'a>(
        &'a self,
        metadata: &BeliefMetadata,
        other: &'a Literal<NonGround>,
        existing_bindings: Option<&'a Bindings>,
    ) -> unification::Result<Bindings<'a>> {
        use core::ops::BitXor;

        let (negated, structure) = {
            let (negated, structure) = self.0.as_literal_atom();
            (negated.bitxor(metadata.negated), structure)
        };

        match (negated, other) {
            (n1, Literal::Atom { negated: n2, .. }) if n1 != *n2 => {
                Err(UnificationFailedError::NegationMismatch)
            }
            (
                _,
                Literal::Atom {
                    negated: _,
                    structure: s2,
                },
            ) => structure.unify(s2, existing_bindings),
            (_, Literal::Variable(NonGround(_))) => unimplemented!(),
        }
    }
}

#[derive(Debug, Default)]
struct BeliefMetadata {
    negated: bool,
}

impl BeliefMetadata {
    /// Updates this metadata with the given one, attempting to maintain consistency.
    fn update(&mut self, other: Self) -> bool {
        let mut updated = false;
        if self.negated != other.negated {
            self.negated = other.negated;
            updated = true;
        }
        updated
    }

    /// Removes this metadata from the stored data. Returns `true` for the first element
    /// if it gave an update, and returns `true` for the second if the complete entry requires
    /// removing.
    fn remove(&mut self, other: Self) -> (bool, bool) {
        let (updated, mut should_remove) = (false, false);
        if self.negated == other.negated {
            should_remove = true;
        }
        (updated, should_remove)
    }
}

mod query {
    use alloc::collections::btree_map::Iter;

    use crate::bindings::Bindings;
    use crate::literal::Literal;

    use super::{BeliefMetadata, NormalizedBelief};

    /// Lazy resolution of a literal query.
    pub struct Query<'a> {
        beliefs: Option<Iter<'a, NormalizedBelief, BeliefMetadata>>,
        query: &'a Literal,
    }

    impl<'a> Query<'a> {
        pub(super) fn new(
            beliefs: Option<Iter<'a, NormalizedBelief, BeliefMetadata>>,
            query: &'a Literal,
        ) -> Self {
            Self { beliefs, query }
        }

        pub fn next_bindings<'b>(
            &mut self,
            existing_bindings: Option<&'a Bindings<'b>>,
        ) -> Option<Bindings<'a>> {
            self.beliefs
                .as_mut()?
                .find_map(|(b, m)| b.unify_literal(m, self.query, existing_bindings).ok())
        }
    }
}

fn atom_and_arity<G: Clone>(structure: &Structure<G>) -> (Atom, usize) {
    (
        structure.functor.clone(),
        structure
            .arguments
            .as_ref()
            .map(|args| args.len())
            .unwrap_or(0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    mod query {
        use alloc::boxed::Box;
        use alloc::vec;
        use alloc::vec::Vec;

        use crate::bindings::TermView;
        use crate::literal::Literal;
        use crate::term::{Atom, Ground, NonGround, Structure, Term};
        use crate::variable::Variable;

        use super::*;

        // --- Fixed Helper Functions ---

        fn str_term(s: &str) -> Term<NonGround> {
            Term::String(s.into())
        }

        fn num_term(n: f32) -> Term<NonGround> {
            Term::Number(n.into())
        }

        fn var() -> Variable {
            Variable::new()
        }

        fn var_term(variable: &Variable) -> Term<NonGround> {
            Term::Variable(NonGround(variable.clone()))
        }

        /// Creates a Belief from a simple functor and string arguments safely
        fn make_belief(functor: &str, args: Vec<&str>, negated: bool) -> Belief {
            let terms = args
                .into_iter()
                .map(|s| Term::<Ground>::String(s.into()))
                .collect::<Vec<_>>()
                .into_boxed_slice();
            let lit = Literal::Atom {
                negated,
                structure: Structure {
                    functor: Atom(functor.into()),
                    arguments: if terms.is_empty() { None } else { Some(terms) },
                },
            };
            Belief::from_ground_literal(lit)
        }

        // --- Comprehensive Test Suite ---

        #[test]
        fn signature_partitioning() {
            let mut bb = BeliefBase::default();

            bb.assert(make_belief("p", vec!["a"], false));
            bb.assert(make_belief("p", vec!["a", "b"], false));
            bb.assert(make_belief("q", vec!["a"], false));

            let v = var();
            let q_p1 = Literal::Atom {
                negated: false,
                structure: Structure {
                    functor: Atom("p".into()),
                    arguments: Some(Box::new([var_term(&v)])),
                },
            };

            let mut query = bb.query(&q_p1);
            assert!(query.next_bindings(None).is_some());
            assert!(
                query.next_bindings(None).is_none(),
                "Should only match p/1 signature"
            );
        }

        #[test]
        fn negation_consistency_and_replacement() {
            let mut bb = BeliefBase::default();
            let f = "weather";
            let a = "sunny";

            // Assert weather(sunny)
            bb.assert(make_belief(f, vec![a], false));

            let q_pos = Literal::Atom {
                negated: false,
                structure: Structure {
                    functor: Atom(f.into()),
                    arguments: Some(Box::new([str_term(a)])),
                },
            };
            assert!(bb.query(&q_pos).next_bindings(None).is_some());

            // Assert ~weather(sunny) - should update the collection
            bb.assert(make_belief(f, vec![a], true));

            assert!(
                bb.query(&q_pos).next_bindings(None).is_none(),
                "Positive belief should be superseded"
            );

            let q_neg = Literal::Atom {
                negated: true,
                structure: Structure {
                    functor: Atom(f.into()),
                    arguments: Some(Box::new([str_term(a)])),
                },
            };
            assert!(bb.query(&q_neg).next_bindings(None).is_some());
        }

        #[test]
        fn multi_result_unification() {
            let mut bb = BeliefBase::default();
            bb.assert(make_belief("likes", vec!["alice", "pizza"], false));
            bb.assert(make_belief("likes", vec!["alice", "sushi"], false));

            let v = var();
            let query = Literal::Atom {
                negated: false,
                structure: Structure {
                    functor: Atom("likes".into()),
                    arguments: Some(Box::new([str_term("alice"), var_term(&v)])),
                },
            };

            let mut q = bb.query(&query);
            let mut found = Vec::new();
            while let Some(bindings) = q.next_bindings(None) {
                if let Some(TermView::Term(Term::String(s))) = bindings.get(&v) {
                    found.push(s.clone());
                }
            }

            assert_eq!(found.len(), 2);
            assert!(found.contains(&"pizza".into()));
            assert!(found.contains(&"sushi".into()));
        }

        #[test]
        fn deep_structural_query() {
            let mut bb = BeliefBase::default();

            // Manual construction of nested belief: at(robot, pos(10, 20))
            let inner_struct = Structure {
                functor: Atom("pos".into()),
                arguments: Some(Box::new([
                    Term::<Ground>::Number(10.0.into()),
                    Term::<Ground>::Number(20.0.into()),
                ])),
            };
            let outer_lit = Literal::Atom {
                negated: false,
                structure: Structure {
                    functor: Atom("at".into()),
                    arguments: Some(Box::new([
                        Term::<Ground>::String("robot".into()),
                        Term::<Ground>::Literal {
                            negated: false,
                            structure: inner_struct,
                        },
                    ])),
                },
            };
            bb.assert(Belief::from_ground_literal(outer_lit));

            let v_x = var();
            let query_inner = Structure {
                functor: Atom("pos".into()),
                arguments: Some(Box::new([var_term(&v_x), num_term(20.0)])),
            };
            let query = Literal::Atom {
                negated: false,
                structure: Structure {
                    functor: Atom("at".into()),
                    arguments: Some(Box::new([
                        str_term("robot"),
                        Term::Literal {
                            negated: false,
                            structure: query_inner,
                        },
                    ])),
                },
            };

            let bindings = bb
                .query(&query)
                .next_bindings(None)
                .expect("Deep unification failed");
            assert_eq!(bindings.get(&v_x), Some(&num_term(10.0).as_view()));
        }

        #[test]
        fn arity_zero_and_empty_states() {
            let mut bb = BeliefBase::default();

            // p.
            let b_p = Literal::Atom {
                negated: false,
                structure: Structure {
                    functor: Atom("p".into()),
                    arguments: None,
                },
            };
            bb.assert(Belief::from_ground_literal(b_p));

            let q_p = Literal::Atom {
                negated: false,
                structure: Structure {
                    functor: Atom("p".into()),
                    arguments: None,
                },
            };
            assert!(bb.query(&q_p).next_bindings(None).is_some());

            // Ensure it doesn't match p(X)
            let v = var();
            let q_p1 = Literal::Atom {
                negated: false,
                structure: Structure {
                    functor: Atom("p".into()),
                    arguments: Some(vec![var_term(&v)].into_boxed_slice()),
                },
            };
            assert!(bb.query(&q_p1).next_bindings(None).is_none());
        }

        #[test]
        fn assertion_redundancy() {
            let mut bb = BeliefBase::default();
            let b = make_belief("fact", vec!["shared"], false);

            assert!(bb.assert(b.clone()));
            assert!(
                !bb.assert(b.clone()),
                "Duplicate assert should return false"
            );

            let q = Literal::Atom {
                negated: false,
                structure: Structure {
                    functor: Atom("fact".into()),
                    arguments: Some(Box::new([str_term("shared")])),
                },
            };

            let mut query = bb.query(&q);
            assert!(query.next_bindings(None).is_some());
            assert!(
                query.next_bindings(None).is_none(),
                "Should only yield one result for redundant facts"
            );
        }

        #[test]
        fn removal_logic() {
            let mut bb = BeliefBase::default();
            let b = make_belief("temp", vec!["val"], false);

            bb.assert(b.clone());
            assert!(bb.remove(b.clone()));
            assert!(!bb.remove(b), "Second removal should return false");

            let q = Literal::Atom {
                negated: false,
                structure: Structure {
                    functor: Atom("temp".into()),
                    arguments: Some(Box::new([str_term("val")])),
                },
            };
            assert!(bb.query(&q).next_bindings(None).is_none());
        }
    }
}
