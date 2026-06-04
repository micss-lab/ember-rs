use alloc::vec::Vec;

use crate::literal::Literal;
use crate::term::Structure;
use crate::term::view::{StructureView, TermView};
use crate::term::{NonGround, Term};

use super::BindingLookup;

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
            TermView::Literal { negated, structure } => Self::Atom {
                negated,
                structure: structure.to_owned(),
            },
            TermView::Variable(v) => Self::Variable(NonGround(v.clone())),

            _ => return Err(ResolveFailure::IncorrectKind),
        })
    }

    fn resolve_as_view<'b>(
        &'b self,
        bindings: &'b impl BindingLookup,
    ) -> Result<TermView<'b>, ResolveFailure> {
        Ok(match *self {
            Literal::Atom {
                negated,
                ref structure,
            } => TermView::Literal {
                negated,
                structure: structure.resolve_as_view(bindings)?,
            },
            Literal::Variable(NonGround(ref v)) => {
                bindings.lookup(v).unwrap_or(TermView::Variable(v))
            }
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
            Term::Variable(NonGround(ref v)) => bindings.lookup(v).unwrap_or(TermView::Variable(v)),
            Term::Literal {
                negated,
                ref structure,
            } => TermView::Literal {
                negated,
                structure: structure.resolve_as_view(bindings)?,
            },
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
    use crate::term::{Atom, NonGround, Structure, Term};

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
    fn test_resolve_unbound_variable_literal_remains_variable() {
        let bindings: Bindings<'_> = Bindings::empty();
        let var = variable();
        let literal = literal_variable(&var);

        let resolved = literal
            .clone()
            .resolve(&bindings)
            .expect("Should resolve successfully");

        assert_eq!(resolved, literal);
        assert!(matches!(resolved, Literal::Variable(_)));
    }

    #[test]
    fn test_resolve_variable_to_atom_literal() {
        let target_atom = literal("sunny", vec![]);
        let target_view = TermView::from(&target_atom);

        let var = variable();
        let bindings = bindings(vec![(var.clone(), target_view)]);
        let literal = literal_variable(&var);

        let resolved = literal
            .resolve(&bindings)
            .expect("Should resolve successfully");

        assert_eq!(resolved, target_atom);
        assert!(matches!(resolved, Literal::Atom { .. }));
    }

    #[test]
    fn test_resolve_variable_to_invalid_kind_fails() {
        let var_num = variable();
        let num_bindings = bindings(vec![(var_num.clone(), TermView::Number(42.0.into()))]);
        let lit_num = literal_variable(&var_num);

        let result_num = lit_num.resolve(&num_bindings);
        assert!(matches!(result_num, Err(ResolveFailure::IncorrectKind)));

        let var_str = variable();
        let term_str = string("hello");
        let str_bindings = bindings(vec![(var_str.clone(), TermView::Term(&term_str))]);
        let lit_str = literal_variable(&var_str);

        let result_str = lit_str.resolve(&str_bindings);
        assert!(matches!(result_str, Err(ResolveFailure::IncorrectKind)));
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
        let literal = Literal::Variable(NonGround(var));

        // Ensure that resolve_possible_as_view captures underlying literal aspects,
        // but note that the `negated` value produced matches the variant wrapped by the view.
        let resolved = literal
            .resolve(&bindings)
            .expect("Should resolve successfully");

        if let Literal::Atom { negated, .. } = resolved {
            assert!(!negated, "Inner ground atom definition was not negated");
        } else {
            panic!("Expected a Literal::Atom");
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
        let embedded_term = Term::Literal {
            negated: false,
            structure: inner_structure,
        };

        // Outer wrapper: outer_lit(embedded_term)
        let outer_literal = literal("outer_lit", vec![embedded_term]);

        let resolved = outer_literal
            .resolve(&bindings)
            .expect("Should recursively map out inner structure terms");

        if let Literal::Atom { structure, .. } = resolved {
            let outer_args = structure.arguments.expect("Should have outer args");
            let Term::Literal {
                structure: inner_struct,
                ..
            } = &outer_args[0]
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
        } else {
            panic!("Expected a Literal::Atom wrapper");
        }
    }
}
