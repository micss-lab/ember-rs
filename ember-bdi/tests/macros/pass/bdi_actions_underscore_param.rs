extern crate alloc;
use ember::agent::bdi::{bdi_actions, bdi_agent};
use ember::agent::bdi::term::Term;

#[bdi_agent(asl = {})]
struct Agent;

#[bdi_actions]
impl Agent {
    fn measure(&mut self, _value: f32) {}
    fn record(&mut self, _key: alloc::string::String, _val: f32) {}
}

fn main() {
    let _ = Agent.into_agent();
    let _ = Agent::measure_action(Term::from(1.0_f32));
    let _ = Agent::record_action(
        Term::from(alloc::string::String::from("k")),
        Term::from(2.0_f32),
    );
}
