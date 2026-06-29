extern crate alloc;
use ember::agent::bdi::literal::{IntoLiteral, Literal};
use ember::agent::bdi::term::{Atom, Structure, Term};

#[derive(IntoLiteral)]
struct Wrapper {
    inner: Literal,
}

fn main() {
    let inner = Literal {
        negated: false,
        structure: Structure {
            functor: Atom("inner".into()),
            arguments: None,
        },
    };
    let lit = Wrapper { inner }.into_literal();
    assert_eq!(lit.structure.functor.0.as_str(), "wrapper");
    let args = lit.structure.arguments.as_ref().unwrap();
    assert_eq!(args.len(), 1);
    assert!(matches!(&args[0], Term::Literal(_)));
}
