extern crate alloc;
use ember::agent::bdi::bdi_agent;

#[bdi_agent(asl = {
    +!brew <- .wait(500, 600).
})]
struct Agent;

fn main() {}
