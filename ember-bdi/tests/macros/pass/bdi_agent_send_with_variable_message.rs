extern crate alloc;
use ember::agent::bdi::{bdi_actions, bdi_agent};

// The `.send` builtin also accepts a bound variable as the message: instead of a literal
// resolved at compile time, the payload is resolved from the bindings at runtime (see
// `VariableOrLiteral`). Also exercises the callback list on the same call.
#[bdi_agent(asl = {
    +notify(Msg) <- .send("watcher@local", "inform", Msg, [on_success(ack)]).
})]
pub struct Agent;

#[bdi_actions]
impl Agent {}

fn main() {
    let _ = Agent.into_agent();
}
