extern crate alloc;
use ember::agent::bdi::bdi_agent;

#[bdi_agent(asl = {
    +!brew <- .wait("500ms").
})]
struct Agent;

fn main() {}
