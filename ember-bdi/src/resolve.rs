use alloc::vec::Vec;

use crate::bindings::BindingLookup;
use crate::literal::Literal;
use crate::term::Structure;
use crate::term::Term;
use crate::term::view::{StructureView, TermView};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResolveFailure {
    /// The structural language type the variable resolved to did not match the context it was
    /// used in. For example, a variable used in the place of a literal should always resolve
    /// to a literal.
    IncorrectKind,
}

impl core::fmt::Display for ResolveFailure {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "resolve failure: {}",
            match self {
                ResolveFailure::IncorrectKind => "incorrect kind",
            }
        )
    }
}

impl core::error::Error for ResolveFailure {}

pub trait Resolve: Sized {
    type View<'a>
    where
        // Bound currently needed until GAT's are fully supported. See issue
        // https://github.com/rust-lang/rust/issues/87479.
        Self: 'a;

    fn resolve(self, bindings: &impl BindingLookup) -> Result<Self, ResolveFailure>;

    fn resolve_as_view<'a>(
        &'a self,
        bindings: &'a impl BindingLookup,
    ) -> Result<Self::View<'a>, ResolveFailure>;
}

impl Resolve for Literal {
    type View<'a> = TermView<'a>;

    /// Resolve the literal using existing bindings as much as possible verifying that the
    /// created binding is valid in the place it used.
    fn resolve(self, bindings: &impl BindingLookup) -> Result<Self, ResolveFailure> {
        Ok(match self.resolve_as_view(bindings)? {
            TermView::Literal { negated, structure } => Self {
                negated,
                structure: structure.to_owned(),
            },

            _ => return Err(ResolveFailure::IncorrectKind),
        })
    }

    fn resolve_as_view<'b>(
        &'b self,
        bindings: &'b impl BindingLookup,
    ) -> Result<TermView<'b>, ResolveFailure> {
        Ok(match *self {
            Literal {
                negated,
                ref structure,
            } => TermView::Literal {
                negated,
                structure: structure.resolve_as_view(bindings)?,
            },
        })
    }
}

impl Resolve for Term {
    type View<'a> = TermView<'a>;

    fn resolve(self, bindings: &impl BindingLookup) -> Result<Self, ResolveFailure> {
        Ok(self.resolve_as_view(bindings)?.to_owned())
    }

    fn resolve_as_view<'a>(
        &'a self,
        bindings: &'a impl BindingLookup,
    ) -> Result<TermView<'a>, ResolveFailure> {
        Ok(match *self {
            Term::Number(_) | Term::String(_) => TermView::Term(self),
            Term::Variable(ref v) => bindings.lookup_view(v).unwrap_or(TermView::Variable(v)),
            Term::Literal(ref literal) => literal.resolve_as_view(bindings)?,
        })
    }
}

impl Resolve for Structure {
    type View<'a> = StructureView<'a>;

    fn resolve(self, bindings: &impl BindingLookup) -> Result<Self, ResolveFailure> {
        Ok(self.resolve_as_view(bindings)?.to_owned())
    }

    fn resolve_as_view<'b>(
        &'b self,
        bindings: &'b impl BindingLookup,
    ) -> Result<StructureView<'b>, ResolveFailure> {
        Ok(StructureView {
            functor: &self.functor,
            arguments: match self.arguments.as_ref() {
                Some(args) => Some(
                    args.into_iter()
                        .map(|a| a.resolve_as_view(bindings))
                        .collect::<Result<Vec<_>, _>>()?
                        .into_boxed_slice(),
                ),
                None => None,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use crate::bindings::Bindings;
    use crate::literal::Literal;
    use crate::term::view::TermView;
    use crate::term::{Atom, Structure, Term};

    use crate::testing::*;

    use super::*;

    #[test]
    fn test_resolve_ground_atom_unchanged() {
        let bindings: Bindings<'_> = Bindings::empty();
        let literal = literal("parent", vec![string("alice"), string("bob")]);

        let resolved = literal
            .clone()
            .resolve(&bindings)
            .expect("Should resolve successfully");

        assert_eq!(resolved, literal);
    }

    #[test]
    fn test_resolve_variable_to_atom_literal() {
        let target_atom = literal("sunny", vec![]);
        let target_view = TermView::from(&target_atom);

        let var = variable();
        let bindings = bindings(vec![(var.clone(), target_view)]);
        let literal = variable_term(&var);

        let resolved = literal
            .resolve(&bindings)
            .expect("Should resolve successfully");

        assert_eq!(resolved, Term::Literal(target_atom));
    }

    #[test]
    fn test_resolve_nested_variables_inside_atom() {
        let (x, y) = (variable(), variable());
        let bindings = bindings(vec![
            (x.clone(), TermView::Number(10.0.into())),
            (y.clone(), TermView::Variable(&x)), // Chained view referencing X
        ]);

        // p(X, Y)
        let literal = literal("p", vec![variable_term(&x), variable_term(&y)]);
        let resolved_view = literal
            .resolve_as_view(&bindings)
            .expect("literal should resolve");

        // Verify structure views are mapped out cleanly
        if let TermView::Literal { structure, .. } = resolved_view {
            let args = structure.arguments.expect("Should contain arguments");
            assert_eq!(args.len(), 2);
            assert_eq!(args[0], TermView::Number(10.0.into()));
            assert_eq!(args[1], TermView::Variable(&x));
        } else {
            panic!("Expected a TermView::Literal");
        }
    }

    #[test]
    fn test_resolve_preserves_negation_states() {
        let target_atom = literal("raining", vec![]);
        let target_view = TermView::from(&target_atom);

        let var = variable();
        let bindings = bindings(vec![(var.clone(), target_view)]);

        // A negated variable literal
        let literal = Term::Variable(var);

        // Ensure that resolve_possible_as_view captures underlying literal aspects,
        // but note that the `negated` value produced matches the variant wrapped by the view.
        let resolved = literal
            .resolve(&bindings)
            .expect("Should resolve successfully");

        if let Term::Literal(Literal { negated, .. }) = resolved {
            assert!(!negated, "Inner ground atom definition was not negated");
        } else {
            panic!("Variable resolved to incorrect term.");
        }
    }

    #[test]
    fn test_deeply_nested_literal_term_resolution() {
        let x = variable();
        let term_num = number(31.5);
        let bindings = bindings(vec![(x.clone(), TermView::Term(&term_num))]);

        // Embedded structure: inner_lit(X)
        let inner_structure = Structure {
            functor: Atom("inner_lit".into()),
            arguments: Some(vec![variable_term(&x)].into_boxed_slice()),
        };

        // Term wrapper around structural literal: Term::Literal { ... }
        let embedded_term = Term::Literal(Literal {
            negated: false,
            structure: inner_structure,
        });

        // Outer wrapper: outer_lit(embedded_term)
        let outer_literal = literal("outer_lit", vec![embedded_term]);

        let resolved = outer_literal
            .resolve(&bindings)
            .expect("Should recursively map out inner structure terms");

        let outer_args = resolved
            .structure
            .arguments
            .expect("Should have outer args");
        let Term::Literal(Literal {
            structure: inner_struct,
            ..
        }) = &outer_args[0]
        else {
            panic!("Expected a nested Term::Literal wrapper");
        };

        let inner_args = inner_struct
            .arguments
            .as_ref()
            .expect("Should have inner args");
        assert_eq!(
            inner_args[0],
            number(31.5),
            "Nested variable X should be resolved to 31.5"
        );
    }
}
