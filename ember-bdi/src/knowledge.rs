use alloc::collections::BTreeMap;

use crate::bindings::Bindings;
use crate::literal::Literal;
use crate::term::unification::{UnificationFailedError, Unify};
use crate::term::{Atom, Ground, NonGround, Structure, Term, unification};

use self::query::{IntoQuery, Query};

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

    pub fn query<'a>(&'a self, query: impl IntoQuery<'a>) -> Query<'a> {
        query.into_query(self)
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
        existing_bindings: Option<&Bindings<'a>>,
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
    use alloc::boxed::Box;
    use alloc::collections::btree_map::Iter;
    use alloc::vec::Vec;

    use crate::bindings::Bindings;
    use crate::literal::Literal;
    use crate::term::NonGround;

    use super::{BeliefBase, BeliefMetadata, NormalizedBelief, atom_and_arity};

    /// Lazy resolution of a literal query.
    #[derive(Debug, Clone)]
    pub struct Query<'a> {
        conjunctions: Box<[Conjunction<'a>]>,
    }

    impl<'a> Query<'a> {
        pub fn next_bindings(&mut self) -> Option<Bindings<'a>> {
            for conjunction in self.conjunctions.iter_mut() {
                let Some(bindings) = conjunction.next_bindings() else {
                    continue;
                };
                return Some(bindings);
            }
            None
        }
    }

    #[derive(Debug, Clone)]
    struct Conjunction<'a> {
        operands: Box<[LiteralQuery<'a>]>,
    }

    impl<'a> Conjunction<'a> {
        fn next_bindings(&mut self) -> Option<Bindings<'a>> {
            let mut current_bindings = Vec::with_capacity(self.operands.len());
            let mut cursor = 0_usize;
            while let Some(operand) = self.operands.get_mut(cursor) {
                match operand.next_bindings(current_bindings.get(cursor.saturating_sub(1))) {
                    Some(bindings) => {
                        current_bindings.push(bindings);
                        cursor += 1;
                    }
                    None => {
                        if cursor == 0 {
                            break;
                        }
                        current_bindings.pop();
                        operand.reset();
                        cursor -= 1;
                    }
                }
            }

            // Reset every operand except the first one in case this function gets
            // called again.
            self.operands.iter_mut().skip(1).for_each(|o| o.reset());

            current_bindings.pop()
        }
    }

    #[derive(Debug, Clone)]
    struct LiteralQuery<'a> {
        /// Closed-world principle of "not". If the query is not satisfyable with
        /// any bindings, it succeeds.
        negated: bool,
        beliefs: Option<Iter<'a, NormalizedBelief, BeliefMetadata>>,
        literal: &'a Literal,

        /// On backtracking, the beliefs it has already tried have to be redone.
        original: Option<Iter<'a, NormalizedBelief, BeliefMetadata>>,
    }

    impl<'a> LiteralQuery<'a> {
        fn next_bindings(
            &mut self,
            existing_bindings: Option<&Bindings<'a>>,
        ) -> Option<Bindings<'a>> {
            match (
                self.negated,
                self.beliefs.as_mut().and_then(|b| {
                    b.find_map(|(b, m)| b.unify_literal(m, self.literal, existing_bindings).ok())
                }),
            ) {
                (false, r) => r,
                (true, Some(_)) => None,
                (true, None) => Some(
                    // Ensure that empty bindings are always returned such that the
                    // query does not fail.
                    existing_bindings
                        .cloned()
                        .unwrap_or_else(Bindings::empty),
                ),
            }
        }

        fn reset(&mut self) {
            self.beliefs = self.original.clone();
        }
    }

    pub trait IntoQuery<'a>
    where
        Self: 'a,
    {
        fn into_query(self, knowledge: &'a BeliefBase) -> Query<'a>;
    }

    impl<'a> IntoQuery<'a> for &'a Literal {
        fn into_query(self, knowledge: &'a BeliefBase) -> Query<'a> {
            let beliefs = match self {
                Literal::Atom { structure, .. } => knowledge
                    .beliefs
                    .get(&atom_and_arity(structure))
                    .map(|b| b.0.iter()),
                Literal::Variable(NonGround(_)) => {
                    unimplemented!("a single variable as a query is not supported (yet)")
                }
            };
            Query {
                conjunctions: Box::new([Conjunction {
                    operands: Box::new([LiteralQuery {
                        negated: false,
                        original: beliefs.clone(),
                        beliefs,
                        literal: self,
                    }]),
                }]),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use alloc::boxed::Box;
        use alloc::vec;
        use alloc::vec::Vec;

        use crate::knowledge::{Belief, BeliefBase};
        use crate::literal::Literal;
        use crate::term::{Atom, Ground, NonGround, Structure, Term};
        use crate::variable::Variable;

        use super::{Conjunction, LiteralQuery, Query};

        // --- Fixed Helper Functions ---

        fn str_term(s: &str) -> Term<NonGround> {
            Term::String(s.into())
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

        #[test]
        fn shared_variable_conjunction() {
            let mut bb = BeliefBase::default();

            // Setup: parent(alice, bob), parent(bob, charlie)
            bb.assert(make_belief("parent", vec!["alice", "bob"], false));
            bb.assert(make_belief("parent", vec!["bob", "charlie"], false));

            let x = var();
            let y = var();

            // Goal: parent(alice, X), parent(X, Y)
            let lit1 = Literal::Atom {
                negated: false,
                structure: Structure {
                    functor: Atom("parent".into()),
                    arguments: Some(Box::new([str_term("alice"), var_term(&x)])),
                },
            };

            let lit2 = Literal::Atom {
                negated: false,
                structure: Structure {
                    functor: Atom("parent".into()),
                    arguments: Some(Box::new([var_term(&x), var_term(&y)])),
                },
            };

            // Manually constructing a conjunction query as IntoQuery is currently &Literal only
            let mut query = Query {
                conjunctions: Box::new([Conjunction {
                    operands: Box::new([
                        LiteralQuery {
                            negated: false,
                            beliefs: bb
                                .beliefs
                                .get(&(Atom("parent".into()), 2))
                                .map(|b| b.0.iter()),
                            original: bb
                                .beliefs
                                .get(&(Atom("parent".into()), 2))
                                .map(|b| b.0.iter()),
                            literal: &lit1,
                        },
                        LiteralQuery {
                            negated: false,
                            beliefs: bb
                                .beliefs
                                .get(&(Atom("parent".into()), 2))
                                .map(|b| b.0.iter()),
                            original: bb
                                .beliefs
                                .get(&(Atom("parent".into()), 2))
                                .map(|b| b.0.iter()),
                            literal: &lit2,
                        },
                    ]),
                }]),
            };

            let bindings = query
                .next_bindings()
                .expect("Should find grandparent relation");
            assert_eq!(bindings.get(&x), Some(&str_term("bob").as_view()));
            assert_eq!(bindings.get(&y), Some(&str_term("charlie").as_view()));
        }

        #[test]
        fn backtracking_across_operands() {
            let mut bb = BeliefBase::default();

            // p(1, 10). p(1, 20). q(20, 30).
            // A query for p(1, X), q(X, Y) should skip X=10 and find X=20.
            bb.assert(make_belief("p", vec!["1", "10"], false));
            bb.assert(make_belief("p", vec!["1", "20"], false));
            bb.assert(make_belief("q", vec!["20", "30"], false));

            let x = var();
            let y = var();

            let lit_p = Literal::Atom {
                negated: false,
                structure: Structure {
                    functor: Atom("p".into()),
                    arguments: Some(Box::new([str_term("1"), var_term(&x)])),
                },
            };
            let lit_q = Literal::Atom {
                negated: false,
                structure: Structure {
                    functor: Atom("q".into()),
                    arguments: Some(Box::new([var_term(&x), var_term(&y)])),
                },
            };

            let mut query = Query {
                conjunctions: Box::new([Conjunction {
                    operands: Box::new([
                        LiteralQuery {
                            negated: false,
                            beliefs: bb.beliefs.get(&(Atom("p".into()), 2)).map(|b| b.0.iter()),
                            original: bb.beliefs.get(&(Atom("p".into()), 2)).map(|b| b.0.iter()),
                            literal: &lit_p,
                        },
                        LiteralQuery {
                            negated: false,
                            beliefs: bb.beliefs.get(&(Atom("q".into()), 2)).map(|b| b.0.iter()),
                            original: bb.beliefs.get(&(Atom("q".into()), 2)).map(|b| b.0.iter()),
                            literal: &lit_q,
                        },
                    ]),
                }]),
            };

            let bindings = query
                .next_bindings()
                .expect("Backtracking should find X=20");
            assert_eq!(bindings.get(&x), Some(&str_term("20").as_view()));
            assert_eq!(bindings.get(&y), Some(&str_term("30").as_view()));
        }

        #[test]
        fn closed_world_negation_success() {
            let mut bb = BeliefBase::default();
            bb.assert(make_belief("is_raining", vec![], false));

            // Query: ~is_sunny (Should succeed because is_sunny is not known)
            let lit_sunny = Literal::Atom {
                negated: false,
                structure: Structure {
                    functor: Atom("is_sunny".into()),
                    arguments: None,
                },
            };

            let mut query = Query {
                conjunctions: Box::new([Conjunction {
                    operands: Box::new([LiteralQuery {
                        negated: true, // Logical NOT
                        beliefs: None,
                        original: None,
                        literal: &lit_sunny,
                    }]),
                }]),
            };

            assert!(
                query.next_bindings().is_some(),
                "Negation of unknown fact should succeed"
            );
        }

        #[test]
        fn nested_structure_extraction() {
            let mut bb = BeliefBase::default();

            // color(circle(red))
            let inner = Structure {
                functor: Atom("circle".into()),
                arguments: Some(Box::new([Term::String("red".into())])),
            };
            bb.assert(Belief::from_ground_literal(Literal::Atom {
                negated: false,
                structure: Structure {
                    functor: Atom("color".into()),
                    arguments: Some(Box::new([Term::Literal {
                        negated: false,
                        structure: inner,
                    }])),
                },
            }));

            let x = var();
            let query_lit = Literal::Atom {
                negated: false,
                structure: Structure {
                    functor: Atom("color".into()),
                    arguments: Some(Box::new([Term::Literal {
                        negated: false,
                        structure: Structure {
                            functor: Atom("circle".into()),
                            arguments: Some(Box::new([var_term(&x)])),
                        },
                    }])),
                },
            };

            let bindings = bb
                .query(&query_lit)
                .next_bindings()
                .expect("Nested unification failed");
            assert_eq!(bindings.get(&x), Some(&str_term("red").as_view()));
        }

        #[test]
        fn partial_structural_mismatch() {
            let mut bb = BeliefBase::default();
            bb.assert(make_belief("pair", vec!["a", "b"], false));

            // Query: pair(a, c) - should fail
            let query_lit = Literal::Atom {
                negated: false,
                structure: Structure {
                    functor: Atom("pair".into()),
                    arguments: Some(Box::new([str_term("a"), str_term("c")])),
                },
            };

            assert!(bb.query(&query_lit).next_bindings().is_none());
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
