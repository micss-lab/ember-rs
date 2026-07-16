extern crate alloc;
use ember::agent::bdi::event::Trigger;
use ember::agent::bdi::literal::IntoLiteral;
use ember::agent::bdi::sensor::Percept;

#[derive(IntoLiteral, Percept)]
#[ember(remove)]
struct GoneReading {
    sensor_id: f32,
}

fn main() {
    let beliefs: alloc::vec::Vec<_> = GoneReading { sensor_id: 7.0 }
        .into_beliefs()
        .into_iter()
        .collect();
    assert_eq!(beliefs.len(), 1);
    let (trigger, lit) = &beliefs[0];
    assert!(matches!(trigger, Trigger::Deletion));
    assert_eq!(lit.structure.functor.0.as_str(), "gone_reading");
}
