extern crate alloc;
use ember::agent::bdi::literal::Literal;
use ember::agent::bdi::term::reference::TermRef;
use ember::agent::bdi::term::{Atom, FromTerm, Structure, Term};

#[derive(FromTerm)]
struct Message {
    text: alloc::string::String,
}

fn main() {
    let term = Term::Literal(Literal {
        negated: false,
        structure: Structure {
            functor: Atom("message".into()),
            arguments: Some(alloc::boxed::Box::new([Term::from(
                alloc::string::String::from("hello world"),
            )])),
        },
    });
    let msg = Message::from_term(TermRef::from(&term)).unwrap();
    assert_eq!(msg.text.as_str(), "hello world");

    let wrong_arity = Term::Literal(Literal {
        negated: false,
        structure: Structure {
            functor: Atom("message".into()),
            arguments: Some(alloc::boxed::Box::new([])),
        },
    });
    assert!(Message::from_term(TermRef::from(&wrong_arity)).is_err());
}
