extern crate alloc;
use ember::agent::bdi::bdi_agent;

#[bdi_agent(asl = {
    +!check(X, Y) <- X = Y < 3; .log("info", "done").
})]
struct Agent;

fn main() {}
