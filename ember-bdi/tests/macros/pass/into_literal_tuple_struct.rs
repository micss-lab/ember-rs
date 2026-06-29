extern crate alloc;
use ember::agent::bdi::literal::IntoLiteral;

#[derive(IntoLiteral)]
struct Wrapper(f32);

fn main() {
    let lit = Wrapper(3.14).into_literal();
    assert_eq!(lit.structure.functor.0.as_str(), "wrapper");
    assert_eq!(lit.structure.arguments.as_ref().map(|a| a.len()), Some(1));
    assert!(!lit.negated);
}
