extern crate alloc;
use alloc::string::String;

use ember::agent::bdi::event::Trigger;
use ember::agent::bdi::literal::IntoLiteral;
use ember::agent::bdi::sensor::Percept;

#[derive(IntoLiteral)]
struct Alarm {
    since: f32,
}

#[derive(Percept)]
enum Event {
    #[ember(add(Alarm { since: opened_at }))]
    Opened { opened_at: f32, reason: String },
}

fn main() {
    let beliefs: alloc::vec::Vec<_> = Event::Opened {
        opened_at: 42.0,
        reason: "test".into(),
    }
    .into_beliefs()
    .into_iter()
    .collect();
    assert_eq!(beliefs.len(), 1);
    assert!(matches!(beliefs[0].0, Trigger::Addition));
    assert_eq!(beliefs[0].1.structure.functor.0.as_str(), "alarm");
    assert_eq!(
        beliefs[0].1.structure.arguments.as_ref().map(|a| a.len()),
        Some(1)
    );
}
