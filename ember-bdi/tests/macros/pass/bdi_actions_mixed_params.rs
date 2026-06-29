extern crate alloc;
use ember::agent::bdi::context::Context;
use ember::agent::bdi::term::Term;
use ember::agent::bdi::{bdi_actions, bdi_agent};

#[bdi_agent(asl = {})]
struct Agent;

#[bdi_actions]
impl Agent {
    fn act(&mut self, _value: f32, ctx: &mut Context<AgentAction>) {
        let _ = ctx;
    }
}

fn main() {
    let _ = Agent.into_agent();
    let _ = Agent::act_action(Term::from(5.0_f32));
}
