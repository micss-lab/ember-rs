extern crate alloc;
use ember::agent::bdi::literal::IntoLiteral;
use ember::agent::bdi::sensor::Percept;
use ember::agent::bdi::{bdi_actions, bdi_agent};

#[derive(IntoLiteral, Percept)]
struct MyPercept;

#[bdi_agent(asl = {}, percept_type = MyPercept, percept_type = MyPercept)]
struct Agent;

#[bdi_actions]
impl Agent {}

fn main() {}
