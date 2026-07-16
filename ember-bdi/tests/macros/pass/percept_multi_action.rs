extern crate alloc;
use ember::agent::bdi::event::Trigger;
use ember::agent::bdi::literal::IntoLiteral;
use ember::agent::bdi::sensor::Percept;

#[derive(IntoLiteral)]
struct NewState(f32);

#[derive(IntoLiteral)]
struct OldState;

#[derive(Percept)]
#[ember(add(NewState(self.value)), remove(OldState))]
struct Transition {
    value: f32,
}

fn main() {
    let beliefs: alloc::vec::Vec<_> = Transition { value: 3.5 }
        .into_beliefs()
        .into_iter()
        .collect();
    assert_eq!(beliefs.len(), 2);
    assert!(matches!(beliefs[0].0, Trigger::Addition));
    assert_eq!(beliefs[0].1.structure.functor.0.as_str(), "new_state");
    assert!(matches!(beliefs[1].0, Trigger::Deletion));
    assert_eq!(beliefs[1].1.structure.functor.0.as_str(), "old_state");
}
