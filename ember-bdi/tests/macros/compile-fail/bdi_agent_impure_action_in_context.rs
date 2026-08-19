extern crate alloc;
use ember::agent::bdi::bdi_agent;

#[bdi_agent(asl = {
    +!brew : .wait(1000) <- .log("info", "done").
})]
struct Agent;

fn main() {}
