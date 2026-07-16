extern crate alloc;
use ember::agent::bdi::event::Trigger;
use ember::agent::bdi::literal::IntoLiteral;
use ember::agent::bdi::sensor::Percept;

#[derive(IntoLiteral)]
struct DoorOpen;

#[derive(Percept)]
enum Door {
    #[ember(add(DoorOpen))]
    Opened,
    #[ember(remove(DoorOpen))]
    Closed,
}

fn main() {
    let opened: alloc::vec::Vec<_> = Door::Opened.into_beliefs().into_iter().collect();
    assert_eq!(opened.len(), 1);
    assert!(matches!(opened[0].0, Trigger::Addition));
    assert_eq!(opened[0].1.structure.functor.0.as_str(), "door_open");

    let closed: alloc::vec::Vec<_> = Door::Closed.into_beliefs().into_iter().collect();
    assert_eq!(closed.len(), 1);
    assert!(matches!(closed[0].0, Trigger::Deletion));
    assert_eq!(closed[0].1.structure.functor.0.as_str(), "door_open");
}
