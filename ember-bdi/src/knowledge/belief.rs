use crate::bindings::Bindings;
use crate::literal::Literal;
use crate::term::{Atom, Ground, NonGround, Structure, Term};
use crate::unification;
use crate::unification::error::UnificationError;

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
    pub(super) Term<NonGround>,
);

impl From<Literal<Ground>> for Belief {
    fn from(literal: Literal<Ground>) -> Self {
        Self::from_ground_literal(literal)
    }
}

impl Belief {
    fn from_ground_literal(literal: Literal<Ground>) -> Self {
        Self(literal.into_non_ground().into())
    }

    /// Normalize the belief into a version that is more efficient when looking up both the
    /// positive and negative variants.
    pub(super) fn normalize(self) -> (NormalizedBelief, BeliefMetadata) {
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
    pub(super) fn atom_and_arity(&self) -> (Atom, usize) {
        let (_, structure) = self.as_literal_atom();
        structure.atom_and_arity()
    }
}

/// A normalized belief is a structure storing an easy look-up version of a belief. For example, a
/// top-level literal is always positive. This allows O(logn) lookup in BTrees even when the belief
/// is negated. To reconstruct the original belief, a [`BeliefMetadata`] instance is required.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct NormalizedBelief(Belief);

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

    pub(super) fn unify_literal<'a>(
        &'a self,
        metadata: &BeliefMetadata,
        other: &'a Literal<NonGround>,
        existing_bindings: Option<&Bindings<'a>>,
    ) -> unification::Result<Bindings<'a>> {
        use crate::unification::traits::Unify;
        use core::ops::BitXor;

        let (negated, structure) = {
            let (negated, structure) = self.0.as_literal_atom();
            (negated.bitxor(metadata.negated), structure)
        };

        match (negated, other) {
            (n1, Literal::Atom { negated: n2, .. }) if n1 != *n2 => {
                Err(UnificationError::NegationMismatch)
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
pub(super) struct BeliefMetadata {
    negated: bool,
}

impl BeliefMetadata {
    /// Updates this metadata with the given one, attempting to maintain consistency.
    pub(super) fn update(&mut self, other: Self) -> bool {
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
    pub(super) fn remove(&mut self, other: Self) -> (bool, bool) {
        let (updated, mut should_remove) = (false, false);
        if self.negated == other.negated {
            should_remove = true;
        }
        (updated, should_remove)
    }
}
