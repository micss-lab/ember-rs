extern crate alloc;
use ember::agent::bdi::literal::IntoLiteral;
use ember::agent::bdi::sensor::Percept;

#[derive(IntoLiteral, Percept)]
struct Bad {
    #[ember(remove)]
    value: f32,
}

fn main() {}
