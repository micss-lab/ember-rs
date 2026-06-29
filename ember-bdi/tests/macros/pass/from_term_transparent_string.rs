extern crate alloc;
use ember::agent::bdi::term::reference::TermRef;
use ember::agent::bdi::term::{FromTerm, Term};

#[derive(FromTerm)]
#[ember(transparent)]
struct Label(alloc::string::String);

fn main() {
    let term = Term::from(alloc::string::String::from("hello"));
    let label = Label::from_term(TermRef::from(&term)).unwrap();
    assert_eq!(label.0.as_str(), "hello");

    let number = Term::from(42.0_f32);
    assert!(Label::from_term(TermRef::from(&number)).is_err());
}
