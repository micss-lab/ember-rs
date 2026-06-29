extern crate alloc;
use ember::agent::bdi::bdi_actions;
use ember::agent::bdi::term::Term;

struct Agent;

#[bdi_actions]
impl Agent {
    fn measure(&mut self, value: f32) {
        let _ = value;
    }
}

fn main() {
    let action = Agent::measure_action(Term::from(42.0_f32));
    let _ = matches!(action, AgentAction::Measure { .. });
}
