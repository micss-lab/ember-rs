extern crate alloc;
use ember::agent::bdi::literal::IntoLiteral;

#[derive(IntoLiteral)]
enum Command {
    Move { x: f32, y: f32 },
    Stop,
}

fn main() {
    let mv = Command::Move { x: 1.0, y: -2.0 }.into_literal();
    assert_eq!(mv.structure.functor.0.as_str(), "move");
    assert_eq!(mv.structure.arguments.as_ref().map(|a| a.len()), Some(2));

    let stop = Command::Stop.into_literal();
    assert_eq!(stop.structure.functor.0.as_str(), "stop");
    assert!(stop.structure.arguments.is_none());
}
