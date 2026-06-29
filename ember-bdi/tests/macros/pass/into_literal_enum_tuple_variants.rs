extern crate alloc;
use ember::agent::bdi::literal::IntoLiteral;

#[derive(IntoLiteral)]
enum Shape {
    Circle(f32),
    Rect(f32, f32),
    Dot,
}

fn main() {
    let circle = Shape::Circle(1.0).into_literal();
    assert_eq!(circle.structure.functor.0.as_str(), "circle");
    assert_eq!(circle.structure.arguments.as_ref().map(|a| a.len()), Some(1));

    let rect = Shape::Rect(2.0, 3.0).into_literal();
    assert_eq!(rect.structure.functor.0.as_str(), "rect");
    assert_eq!(rect.structure.arguments.as_ref().map(|a| a.len()), Some(2));

    let dot = Shape::Dot.into_literal();
    assert_eq!(dot.structure.functor.0.as_str(), "dot");
    assert!(dot.structure.arguments.is_none());
}
