extern crate alloc;
use ember::agent::bdi::bdi_actions;

struct Agent;

#[bdi_actions]
impl Agent {
    #[bdi_action(name = "custom_action")]
    fn my_method(&mut self) {}
}

fn main() {
    let action = Agent::custom_action_action();
    let _ = matches!(action, AgentAction::CustomAction {});
}
