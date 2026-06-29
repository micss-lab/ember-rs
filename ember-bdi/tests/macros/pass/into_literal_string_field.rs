extern crate alloc;
use ember::agent::bdi::literal::IntoLiteral;

#[derive(IntoLiteral)]
struct Label(alloc::string::String);

#[derive(IntoLiteral)]
struct Named {
    name: alloc::string::String,
    value: f32,
}

fn main() {
    let lit = Label("hello".into()).into_literal();
    assert_eq!(lit.structure.functor.0.as_str(), "label");
    assert_eq!(lit.structure.arguments.as_ref().map(|a| a.len()), Some(1));

    let named = Named {
        name: "sensor".into(),
        value: 42.0,
    }
    .into_literal();
    assert_eq!(named.structure.functor.0.as_str(), "named");
    assert_eq!(named.structure.arguments.as_ref().map(|a| a.len()), Some(2));
}
