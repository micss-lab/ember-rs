extern crate alloc;
use ember::agent::bdi::term::FromTerm;

#[derive(FromTerm)]
#[ember(transparent)]
struct Empty;

fn main() {}
