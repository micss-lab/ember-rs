extern crate alloc;
use ember::agent::bdi::term::FromTerm;

#[derive(FromTerm)]
union Bad {
    x: f32,
    y: u32,
}

fn main() {}
