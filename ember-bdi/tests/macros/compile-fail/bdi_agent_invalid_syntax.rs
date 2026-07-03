extern crate alloc;
use ember::agent::bdi::bdi_agent;

#[bdi_agent(asl = {
    +!start <- .log("info", "start")
})]
struct Agent;

fn main() {}
