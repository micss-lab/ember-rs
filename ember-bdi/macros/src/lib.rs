use proc_macro::TokenStream;
use syn::{DeriveInput, ItemImpl, parse_macro_input};

mod ast;
mod macros;
mod token;

struct BdiAgentArgs {
    asl: proc_macro2::TokenStream,
    percept_type: Option<syn::Type>,
}

#[proc_macro_attribute]
pub fn bdi_agent(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as BdiAgentArgs);
    let input = parse_macro_input!(input as DeriveInput);

    macros::agent::expand(args, input).into()
}

#[proc_macro_attribute]
pub fn bdi_actions(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemImpl);

    match macros::actions::expand(input) {
        Ok(token_stream) => token_stream.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(IntoLiteral)]
pub fn derive_into_literal(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    macros::derive::into_literal::expand(input).into()
}

#[proc_macro_derive(Percept)]
pub fn derive_percept(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    macros::derive::percept::expand(input).into()
}
