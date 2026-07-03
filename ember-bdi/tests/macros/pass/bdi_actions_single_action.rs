extern crate alloc;
use ember::agent::bdi::bdi_actions;

struct Agent;

#[bdi_actions]
impl Agent {
    fn do_it(&mut self) {}
}

fn main() {
    let _ = core::mem::size_of::<AgentAction>();
}
