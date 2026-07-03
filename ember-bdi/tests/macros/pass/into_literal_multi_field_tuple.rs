extern crate alloc;
use ember::agent::bdi::literal::IntoLiteral;

#[derive(IntoLiteral)]
struct Vec3(f32, f32, f32);

fn main() {
    let lit = Vec3(1.0, 2.0, 3.0).into_literal();
    assert_eq!(lit.structure.functor.0.as_str(), "vec3");
    assert_eq!(lit.structure.arguments.as_ref().map(|a| a.len()), Some(3));
}
