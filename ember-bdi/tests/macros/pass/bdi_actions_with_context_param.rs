extern crate alloc;
use ember::agent::bdi::bdi_actions;
use ember::agent::bdi::context::Context;

struct Agent;

#[bdi_actions]
impl Agent {
    fn halt(&mut self, ctx: &mut Context<AgentAction>) {
        ctx.stop_platform();
    }
}

fn main() {
    let _ = core::mem::size_of::<AgentAction>();
}
