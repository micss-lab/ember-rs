extern crate alloc;
use ember::agent::bdi::bdi_agent;

#[bdi_agent(asl = {
    +!start <- .forall(item(X) process(X)).
})]
struct Agent;

fn main() {}
