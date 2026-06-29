extern crate alloc;
use ember::agent::bdi::term::Term;
use ember::agent::bdi::{bdi_actions, bdi_agent};

#[bdi_agent(asl = {})]
struct Agent;

#[bdi_actions]
impl Agent {
    fn move_to(&mut self, _x: f32, _y: f32) {}
}

fn main() {
    let _ = Agent.into_agent();
    let _ = Agent::move_to_action(Term::from(3.0_f32), Term::from(4.0_f32));
}
