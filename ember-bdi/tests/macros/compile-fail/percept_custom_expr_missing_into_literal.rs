extern crate alloc;
use ember::agent::bdi::literal::IntoLiteral;
use ember::agent::bdi::sensor::Percept;

struct NotLiteral;

#[derive(IntoLiteral, Percept)]
#[ember(add(NotLiteral))]
struct Bad;

fn main() {}
