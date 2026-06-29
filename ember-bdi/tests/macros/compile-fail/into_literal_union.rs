extern crate alloc;
use ember::agent::bdi::literal::IntoLiteral;

#[derive(IntoLiteral)]
union MyUnion {
    x: f32,
    y: u32,
}

fn main() {}
