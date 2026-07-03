extern crate alloc;
use ember::agent::bdi::literal::IntoLiteral;

#[derive(IntoLiteral)]
enum Color {
    Red,
    Green,
    Blue,
}

fn main() {
    let red = Color::Red.into_literal();
    assert_eq!(red.structure.functor.0.as_str(), "red");
    assert!(red.structure.arguments.is_none());

    let green = Color::Green.into_literal();
    assert_eq!(green.structure.functor.0.as_str(), "green");
    assert!(green.structure.arguments.is_none());

    let blue = Color::Blue.into_literal();
    assert_eq!(blue.structure.functor.0.as_str(), "blue");
    assert!(blue.structure.arguments.is_none());
}
