extern crate alloc;
use ember::agent::bdi::bdi_actions;

struct Agent;

#[bdi_actions]
impl Agent {
    fn start(&mut self) {}
    fn stop(&mut self) {}
    fn reset(&mut self) {}
}

fn main() {
    let _ = core::mem::size_of::<AgentAction>();
}
