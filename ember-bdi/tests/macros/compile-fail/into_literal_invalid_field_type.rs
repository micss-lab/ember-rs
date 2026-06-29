extern crate alloc;
use ember::agent::bdi::literal::IntoLiteral;

#[derive(IntoLiteral)]
struct Sensor {
    readings: alloc::vec::Vec<f32>,
}

fn main() {}
