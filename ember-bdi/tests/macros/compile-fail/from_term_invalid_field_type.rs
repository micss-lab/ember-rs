extern crate alloc;
use ember::agent::bdi::term::FromTerm;

#[derive(FromTerm)]
struct Sensor {
    readings: alloc::vec::Vec<f32>,
}

fn main() {}
