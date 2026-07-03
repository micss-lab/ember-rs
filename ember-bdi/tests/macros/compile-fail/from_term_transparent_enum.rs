extern crate alloc;
use ember::agent::bdi::term::FromTerm;

#[derive(FromTerm)]
#[ember(transparent)]
enum Direction {
    North,
    South,
}

fn main() {}
