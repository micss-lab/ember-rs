extern crate alloc;
use ember::agent::bdi::event::Trigger;
use ember::agent::bdi::literal::IntoLiteral;
use ember::agent::bdi::sensor::Percept;

#[derive(IntoLiteral)]
struct PowerOnMarker;

#[derive(Percept)]
#[ember(add(PowerOnMarker))]
enum PowerState {
    On,
    Off,
    Standby,
}

fn main() {
    for state in [PowerState::On, PowerState::Off, PowerState::Standby] {
        let beliefs: alloc::vec::Vec<_> = state.into_beliefs().into_iter().collect();
        assert_eq!(beliefs.len(), 1);
        assert!(matches!(beliefs[0].0, Trigger::Addition));
        assert_eq!(beliefs[0].1.structure.functor.0.as_str(), "power_on_marker");
    }
}
