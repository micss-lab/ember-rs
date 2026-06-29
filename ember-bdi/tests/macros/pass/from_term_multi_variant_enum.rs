extern crate alloc;
use ember::agent::bdi::literal::Literal;
use ember::agent::bdi::term::reference::TermRef;
use ember::agent::bdi::term::{Atom, FromTerm, Structure, Term};

#[derive(FromTerm, Debug, PartialEq)]
enum Direction {
    North,
    South,
    East,
    West,
}

fn main() {
    for (name, expected) in [
        ("north", Direction::North),
        ("south", Direction::South),
        ("east", Direction::East),
        ("west", Direction::West),
    ] {
        let term = Term::Literal(Literal {
            negated: false,
            structure: Structure {
                functor: Atom(name.into()),
                arguments: Some(alloc::boxed::Box::new([])),
            },
        });
        assert_eq!(Direction::from_term(TermRef::from(&term)).unwrap(), expected);
    }

    let unknown = Term::Literal(Literal {
        negated: false,
        structure: Structure {
            functor: Atom("up".into()),
            arguments: Some(alloc::boxed::Box::new([])),
        },
    });
    assert!(Direction::from_term(TermRef::from(&unknown)).is_err());
}
