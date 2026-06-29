extern crate alloc;
use ember::agent::bdi::term::FromTerm;

#[derive(FromTerm)]
#[ember(transparent)]
struct Multi(f32, f32);

fn main() {}
