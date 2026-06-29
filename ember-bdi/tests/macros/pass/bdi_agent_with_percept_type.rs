extern crate alloc;
use ember::agent::bdi::literal::IntoLiteral;
use ember::agent::bdi::sensor::Percept;
use ember::agent::bdi::{bdi_actions, bdi_agent};

#[derive(IntoLiteral, Percept)]
struct TemperatureReading;

#[bdi_agent(
    percept_type = TemperatureReading,
    asl = {
        !monitor.
    }
)]
struct SensingAgent;

#[bdi_actions]
impl SensingAgent {}

fn main() {
    let _ = SensingAgent.into_agent();
}
