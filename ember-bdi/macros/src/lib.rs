use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::parse::{Parse, ParseStream};
use syn::{DeriveInput, ItemImpl, Token, parse_macro_input};

use crate::token::FlatTokenStream;

mod action;
mod ast;
mod compiler;
mod parser;
mod token;

#[proc_macro_attribute]
pub fn bdi_agent(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as BdiAgentArgs);
    let input = parse_macro_input!(input as DeriveInput);

    let program = match parser::asl_token_stream::program(&FlatTokenStream::new(args.asl)) {
        Ok(p) => p,
        Err(err) => {
            let msg = format!("expected {}", err.expected);
            return quote_spanned!(err.location.0=> #input compile_error!(#msg);).into();
        }
    };

    let program = compiler::asl::expand(&program, &input.ident);

    quote! {
        #input
        #program
    }
    .into()
}

struct BdiAgentArgs {
    asl: proc_macro2::TokenStream,
}

impl Parse for BdiAgentArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Err(input.error("expected `asl = ...`"));
        }

        let ident: syn::Ident = input.parse()?;
        if ident != "asl" {
            return Err(syn::Error::new_spanned(
                ident,
                "unknown argument, expected `asl = ...`",
            ));
        }

        input.parse::<Token![=]>()?;

        let asl: proc_macro2::TokenStream = input.parse()?;

        if !input.is_empty() {
            return Err(input
                .error("unexpected tokens after `asl = ...`, only a single argument is allowed"));
        }

        Ok(BdiAgentArgs { asl })
    }
}

#[proc_macro_attribute]
pub fn bdi_actions(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemImpl);

    match compiler::actions::expand(input) {
        Ok(token_stream) => token_stream.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
