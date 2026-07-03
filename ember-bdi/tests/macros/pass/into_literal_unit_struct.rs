extern crate alloc;
use ember::agent::bdi::literal::IntoLiteral;

#[derive(IntoLiteral)]
struct Foo;

fn main() {
    let lit = Foo.into_literal();
    assert_eq!(lit.structure.functor.0.as_str(), "foo");
    assert!(lit.structure.arguments.is_none());
    assert!(!lit.negated);
}
