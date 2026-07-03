extern crate alloc;
use ember::agent::bdi::event::Trigger;
use ember::agent::bdi::literal::IntoLiteral;
use ember::agent::bdi::sensor::Percept;

#[derive(IntoLiteral, Percept)]
struct Reading {
    value: f32,
}

fn main() {
    let beliefs: alloc::vec::Vec<_> = Reading { value: 42.0 }.into_beliefs().into_iter().collect();
    assert_eq!(beliefs.len(), 1);
    let (trigger, lit) = &beliefs[0];
    assert!(matches!(trigger, Trigger::Addition));
    assert_eq!(lit.structure.functor.0.as_str(), "reading");
    assert_eq!(lit.structure.arguments.as_ref().map(|a| a.len()), Some(1));
}
