extern crate alloc;
use ember::agent::bdi::event::Trigger;
use ember::agent::bdi::literal::IntoLiteral;
use ember::agent::bdi::sensor::Percept;

#[derive(IntoLiteral)]
struct ClearanceTimer(f32);

#[derive(Percept)]
enum DoorSensor {
    #[ember(add(ClearanceTimer(_0)))]
    Clearance(f32),
}

fn main() {
    let beliefs: alloc::vec::Vec<_> = DoorSensor::Clearance(30.0)
        .into_beliefs()
        .into_iter()
        .collect();
    assert_eq!(beliefs.len(), 1);
    assert!(matches!(beliefs[0].0, Trigger::Addition));
    assert_eq!(beliefs[0].1.structure.functor.0.as_str(), "clearance_timer");
    let arguments = beliefs[0].1.structure.arguments.as_ref().unwrap();
    assert_eq!(arguments.len(), 1);
}
