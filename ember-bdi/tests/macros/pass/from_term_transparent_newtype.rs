extern crate alloc;
use ember::agent::bdi::term::reference::TermRef;
use ember::agent::bdi::term::{FromTerm, Term};

#[derive(FromTerm)]
#[ember(transparent)]
struct Celsius(f32);

fn main() {
    let term = Term::from(100.0_f32);
    let c = Celsius::from_term(TermRef::from(&term)).unwrap();
    assert_eq!(c.0, 100.0_f32);

    let non_number = Term::from(alloc::string::String::from("hot"));
    assert!(Celsius::from_term(TermRef::from(&non_number)).is_err());
}
