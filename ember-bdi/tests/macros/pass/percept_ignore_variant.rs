extern crate alloc;
use ember::agent::bdi::literal::IntoLiteral;
use ember::agent::bdi::sensor::Percept;

#[derive(IntoLiteral, Percept)]
enum Heartbeat {
    #[ember(ignore)]
    Tick,
    Beat,
}

fn main() {
    let ticks: alloc::vec::Vec<_> = Heartbeat::Tick.into_beliefs().into_iter().collect();
    assert!(ticks.is_empty());

    let beats: alloc::vec::Vec<_> = Heartbeat::Beat.into_beliefs().into_iter().collect();
    assert_eq!(beats.len(), 1);
}
