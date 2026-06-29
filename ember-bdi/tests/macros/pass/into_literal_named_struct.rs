extern crate alloc;
use ember::agent::bdi::literal::IntoLiteral;

#[derive(IntoLiteral)]
struct Point {
    x: f32,
    y: f32,
}

fn main() {
    let lit = Point { x: 1.0, y: 2.0 }.into_literal();
    assert_eq!(lit.structure.functor.0.as_str(), "point");
    assert_eq!(lit.structure.arguments.as_ref().map(|a| a.len()), Some(2));
    assert!(!lit.negated);
}
