use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::parse::{Parse, ParseStream};
use syn::{DeriveInput, Token, parse_macro_input};

use crate::token::FlatTokenStream;

mod action;
mod ast;
mod compiler;
mod parser;
mod token;

#[proc_macro_attribute]
pub fn bdi_agent(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let args = match syn::parse::<BdiAgentArgs>(args) {
        Ok(args) => args,
        Err(e) => {
            let err = e.to_compile_error();
            return quote! { #input #err }.into();
        }
    };

    let program = match parser::asl_token_stream::program(&FlatTokenStream::new(args.asl)) {
        Ok(p) => p,
        Err(err) => {
            let msg = format!("expected {}", err.expected);
            return quote_spanned!(err.location.0=> #input compile_error!(#msg);).into();
        }
    };

    let program = compiler::compile_asl(&program, "bdi-agent", &input.ident);

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
