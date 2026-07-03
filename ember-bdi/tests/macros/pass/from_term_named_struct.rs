extern crate alloc;
use ember::agent::bdi::literal::Literal;
use ember::agent::bdi::term::reference::TermRef;
use ember::agent::bdi::term::{Atom, FromTerm, Structure, Term};

#[derive(FromTerm)]
struct Point {
    x: f32,
    y: f32,
}

fn main() {
    let term = Term::Literal(Literal {
        negated: false,
        structure: Structure {
            functor: Atom("point".into()),
            arguments: Some(alloc::boxed::Box::new([
                Term::from(1.0_f32),
                Term::from(2.0_f32),
            ])),
        },
    });
    let p = Point::from_term(TermRef::from(&term)).unwrap();
    assert_eq!(p.x, 1.0_f32);
    assert_eq!(p.y, 2.0_f32);

    let wrong_arity = Term::Literal(Literal {
        negated: false,
        structure: Structure {
            functor: Atom("point".into()),
            arguments: Some(alloc::boxed::Box::new([Term::from(1.0_f32)])),
        },
    });
    assert!(Point::from_term(TermRef::from(&wrong_arity)).is_err());
}
