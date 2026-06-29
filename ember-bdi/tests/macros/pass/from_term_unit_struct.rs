extern crate alloc;
use ember::agent::bdi::literal::Literal;
use ember::agent::bdi::term::reference::TermRef;
use ember::agent::bdi::term::{Atom, FromTerm, Structure, Term};

#[derive(FromTerm)]
struct Empty;

fn main() {
    let term = Term::Literal(Literal {
        negated: false,
        structure: Structure {
            functor: Atom("empty".into()),
            arguments: Some(alloc::boxed::Box::new([])),
        },
    });
    assert!(Empty::from_term(TermRef::from(&term)).is_ok());

    let wrong = Term::Literal(Literal {
        negated: false,
        structure: Structure {
            functor: Atom("other".into()),
            arguments: Some(alloc::boxed::Box::new([])),
        },
    });
    assert!(Empty::from_term(TermRef::from(&wrong)).is_err());
}
