extern crate alloc;
use ember::agent::bdi::literal::Literal;
use ember::agent::bdi::term::reference::TermRef;
use ember::agent::bdi::term::{Atom, FromTerm, Structure, Term};

#[derive(FromTerm, Debug, PartialEq)]
enum Shape {
    Circle(f32),
    Rect(f32, f32),
    Point,
}

fn main() {
    let circle_term = Term::Literal(Literal {
        negated: false,
        structure: Structure {
            functor: Atom("circle".into()),
            arguments: Some(alloc::boxed::Box::new([Term::from(1.0_f32)])),
        },
    });
    assert_eq!(
        Shape::from_term(TermRef::from(&circle_term)).unwrap(),
        Shape::Circle(1.0)
    );

    let rect_term = Term::Literal(Literal {
        negated: false,
        structure: Structure {
            functor: Atom("rect".into()),
            arguments: Some(alloc::boxed::Box::new([Term::from(2.0_f32), Term::from(3.0_f32)])),
        },
    });
    assert_eq!(
        Shape::from_term(TermRef::from(&rect_term)).unwrap(),
        Shape::Rect(2.0, 3.0)
    );

    let point_term = Term::Literal(Literal {
        negated: false,
        structure: Structure {
            functor: Atom("point".into()),
            arguments: Some(alloc::boxed::Box::new([])),
        },
    });
    assert_eq!(
        Shape::from_term(TermRef::from(&point_term)).unwrap(),
        Shape::Point
    );

    let unknown = Term::Literal(Literal {
        negated: false,
        structure: Structure {
            functor: Atom("triangle".into()),
            arguments: Some(alloc::boxed::Box::new([])),
        },
    });
    assert!(Shape::from_term(TermRef::from(&unknown)).is_err());
}
