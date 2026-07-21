extern crate alloc;
use ember::agent::bdi::bdi_agent;

#[bdi_agent(asl = {
    +!brew <- .wait(1.5).
})]
struct Agent;

fn main() {}
