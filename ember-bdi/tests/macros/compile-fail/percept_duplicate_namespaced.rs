extern crate alloc;
use ember::agent::bdi::literal::IntoLiteral;
use ember::agent::bdi::sensor::Percept;

#[derive(IntoLiteral, Percept)]
#[ember(percept(add))]
#[ember(percept(remove))]
struct Bad;

fn main() {}
