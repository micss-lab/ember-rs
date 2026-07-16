extern crate alloc;
use ember::agent::bdi::event::Trigger;
use ember::agent::bdi::literal::IntoLiteral;
use ember::agent::bdi::sensor::Percept;
use ember::agent::bdi::term::FromTerm;

#[derive(IntoLiteral)]
struct Marker;

#[derive(IntoLiteral, Percept, FromTerm)]
#[ember(from_term(transparent), percept(remove(Marker)))]
struct Gone(f32);

fn main() {
    let beliefs: alloc::vec::Vec<_> = Gone(1.0).into_beliefs().into_iter().collect();
    assert_eq!(beliefs.len(), 1);
    assert!(matches!(beliefs[0].0, Trigger::Deletion));
    assert_eq!(beliefs[0].1.structure.functor.0.as_str(), "marker");
}
