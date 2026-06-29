extern crate alloc;
use ember::agent::bdi::literal::Literal;
use ember::agent::bdi::term::reference::TermRef;
use ember::agent::bdi::term::{Atom, FromTerm, Structure, Term};

#[derive(FromTerm, Debug, PartialEq)]
enum Status {
    Active,
    Idle,
}

fn main() {
    let active_term = Term::Literal(Literal {
        negated: false,
        structure: Structure {
            functor: Atom("active".into()),
            arguments: Some(alloc::boxed::Box::new([])),
        },
    });
    assert_eq!(
        Status::from_term(TermRef::from(&active_term)).unwrap(),
        Status::Active
    );

    let idle_term = Term::Literal(Literal {
        negated: false,
        structure: Structure {
            functor: Atom("idle".into()),
            arguments: Some(alloc::boxed::Box::new([])),
        },
    });
    assert_eq!(
        Status::from_term(TermRef::from(&idle_term)).unwrap(),
        Status::Idle
    );

    let unknown_term = Term::Literal(Literal {
        negated: false,
        structure: Structure {
            functor: Atom("running".into()),
            arguments: Some(alloc::boxed::Box::new([])),
        },
    });
    assert!(Status::from_term(TermRef::from(&unknown_term)).is_err());
}
