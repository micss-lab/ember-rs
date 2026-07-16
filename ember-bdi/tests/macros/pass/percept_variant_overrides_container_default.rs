extern crate alloc;
use ember::agent::bdi::event::Trigger;
use ember::agent::bdi::literal::IntoLiteral;
use ember::agent::bdi::sensor::Percept;

#[derive(IntoLiteral)]
struct Marker;

#[derive(Percept)]
#[ember(add(Marker))]
enum Signal {
    Normal,
    #[ember(remove(Marker))]
    Cancelled,
}

fn main() {
    let normal: alloc::vec::Vec<_> = Signal::Normal.into_beliefs().into_iter().collect();
    assert_eq!(normal.len(), 1);
    assert!(matches!(normal[0].0, Trigger::Addition));

    let cancelled: alloc::vec::Vec<_> = Signal::Cancelled.into_beliefs().into_iter().collect();
    assert_eq!(cancelled.len(), 1);
    assert!(matches!(cancelled[0].0, Trigger::Deletion));
}
