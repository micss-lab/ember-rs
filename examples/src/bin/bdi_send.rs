#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use log::info;

use ember::Container;
use ember::agent::bdi::{bdi_actions, bdi_agent};

use ember_examples::setup_example;

setup_example!();

// SenderAgent informs ReceiverAgent that it has `resource(water)` on startup.
// The Aid "receiver-agent@local" is validated at compile time — a malformed Aid
// (e.g. missing '@') would cause a compile error from the proc-macro.
#[bdi_agent(asl = {
    !startup.

    +!startup
      <- .send("receiver-agent@local", "inform", resource(water)).
})]
struct SenderAgent;

#[bdi_actions]
impl SenderAgent {}

// ReceiverAgent reacts to the message belief added by the incoming inform.
// The `.send` builtin wraps the payload in `message(...)` before transmission,
// so incoming beliefs arrive as `message(resource(water))`.
#[bdi_agent(asl = {
    +message(resource(X))
      <- .log("info", "Received resource: ", X);
         .stop_platform().
})]
struct ReceiverAgent;

#[bdi_actions]
impl ReceiverAgent {}

fn example() {
    info!("Starting BDI send example");

    Container::new()
        .with_agent(SenderAgent.into_agent())
        .with_agent(ReceiverAgent.into_agent())
        .start()
        .expect("container encountered an error");

    info!("BDI send example finished");
}
