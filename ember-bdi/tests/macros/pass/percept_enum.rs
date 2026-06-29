extern crate alloc;
use ember::agent::bdi::event::Trigger;
use ember::agent::bdi::literal::IntoLiteral;
use ember::agent::bdi::sensor::Percept;

#[derive(IntoLiteral, Percept)]
enum Event {
    Start,
    Stop,
}

fn main() {
    let beliefs: alloc::vec::Vec<_> = Event::Start.into_beliefs().into_iter().collect();
    assert_eq!(beliefs.len(), 1);
    let (trigger, lit) = &beliefs[0];
    assert!(matches!(trigger, Trigger::Addition));
    assert_eq!(lit.structure.functor.0.as_str(), "start");

    let stop_beliefs: alloc::vec::Vec<_> = Event::Stop.into_beliefs().into_iter().collect();
    let (_, stop_lit) = &stop_beliefs[0];
    assert_eq!(stop_lit.structure.functor.0.as_str(), "stop");
}
