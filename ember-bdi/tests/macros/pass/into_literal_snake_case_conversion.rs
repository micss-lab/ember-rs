extern crate alloc;
use ember::agent::bdi::literal::IntoLiteral;

#[derive(IntoLiteral)]
struct MotionSensor;

#[derive(IntoLiteral)]
enum NetworkEvent {
    PacketReceived,
    ConnectionLost { error_code: f32 },
    RetryAttempted(f32),
}

fn main() {
    let sensor = MotionSensor.into_literal();
    assert_eq!(sensor.structure.functor.0.as_str(), "motion_sensor");
    assert!(sensor.structure.arguments.is_none());

    let recv = NetworkEvent::PacketReceived.into_literal();
    assert_eq!(recv.structure.functor.0.as_str(), "packet_received");
    assert!(recv.structure.arguments.is_none());

    let lost = NetworkEvent::ConnectionLost { error_code: 404.0 }.into_literal();
    assert_eq!(lost.structure.functor.0.as_str(), "connection_lost");
    assert_eq!(lost.structure.arguments.as_ref().map(|a| a.len()), Some(1));

    let retry = NetworkEvent::RetryAttempted(3.0).into_literal();
    assert_eq!(retry.structure.functor.0.as_str(), "retry_attempted");
    assert_eq!(retry.structure.arguments.as_ref().map(|a| a.len()), Some(1));
}
