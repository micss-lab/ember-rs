extern crate alloc;
use ember::agent::bdi::{bdi_actions, bdi_agent};

// The receiver aid (`VariableOrReceiver`) and the message (`VariableOrLiteral`) can each be a
// bound variable independently - confirm the two compose on the same `.send` call.
#[bdi_agent(asl = {
    +notify(Addr, Msg) <- .send(Addr, "inform", Msg).
})]
pub struct Agent;

#[bdi_actions]
impl Agent {}

fn main() {
    let _ = Agent.into_agent();
}
