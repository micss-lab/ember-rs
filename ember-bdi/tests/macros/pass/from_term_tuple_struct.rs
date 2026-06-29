extern crate alloc;
use ember::agent::bdi::literal::Literal;
use ember::agent::bdi::term::reference::TermRef;
use ember::agent::bdi::term::{Atom, FromTerm, Structure, Term};

#[derive(FromTerm)]
struct Pair(f32, f32);

fn main() {
    let term = Term::Literal(Literal {
        negated: false,
        structure: Structure {
            functor: Atom("pair".into()),
            arguments: Some(alloc::boxed::Box::new([
                Term::from(3.0_f32),
                Term::from(4.0_f32),
            ])),
        },
    });
    let p = Pair::from_term(TermRef::from(&term)).unwrap();
    assert_eq!(p.0, 3.0_f32);
    assert_eq!(p.1, 4.0_f32);
}
