extern crate alloc;
use ember::agent::bdi::literal::IntoLiteral;
use ember::agent::bdi::sensor::Percept;
use ember::agent::bdi::{bdi_actions, bdi_agent};

#[derive(IntoLiteral, Percept)]
pub struct TemperatureReading;

#[bdi_agent(
    percept_type = TemperatureReading,
    asl = {
        !monitor.
    }
)]
pub struct SensingAgent;

#[bdi_actions]
impl SensingAgent {}

fn main() {
    let _ = SensingAgent.into_agent();
}
