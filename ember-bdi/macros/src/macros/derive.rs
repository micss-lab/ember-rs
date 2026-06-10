pub(crate) mod into_literal {
    //! AI-generated human verified.

    use heck::ToSnakeCase;
    use proc_macro2::TokenStream;
    use quote::quote;
    use syn::{Data, DeriveInput, Fields};

    pub(crate) fn expand(input: DeriveInput) -> TokenStream {
        let name = &input.ident;
        let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

        let body = match &input.data {
            Data::Struct(data) => {
                let functor = name.to_string().to_snake_case();
                expand_fields(&functor, &data.fields)
            }
            Data::Enum(data) => {
                let variants = data.variants.iter().map(|v| {
                    let variant_name = &v.ident;
                    let functor = variant_name.to_string().to_snake_case();
                    let field_expansion = expand_fields_for_variant(&functor, &v.fields);

                    match &v.fields {
                        Fields::Named(fields) => {
                            let field_names = fields.named.iter().map(|f| &f.ident);
                            quote! {
                                Self::#variant_name { #(#field_names),* } => #field_expansion
                            }
                        }
                        Fields::Unnamed(fields) => {
                            let field_names = (0..fields.unnamed.len()).map(|i| {
                                syn::Ident::new(&format!("f{}", i), proc_macro2::Span::call_site())
                            });
                            quote! {
                                Self::#variant_name( #(#field_names),* ) => #field_expansion
                            }
                        }
                        Fields::Unit => {
                            quote! {
                                Self::#variant_name => #field_expansion
                            }
                        }
                    }
                });

                quote! {
                    match self {
                        #(#variants,)*
                    }
                }
            }
            Data::Union(_) => {
                return quote! { compile_error!("IntoLiteral cannot be derived for unions") };
            }
        };

        quote! {
            impl #impl_generics ::ember::agent::bdi::literal::IntoLiteral for #name #ty_generics #where_clause {
                fn into_literal(self) -> ::ember::agent::bdi::literal::Literal<::ember::agent::bdi::term::Ground> {
                    #body
                }
            }
        }
    }

    fn expand_fields(functor: &str, fields: &Fields) -> TokenStream {
        let args = match fields {
            Fields::Named(fields) => {
                let exprs = fields.named.iter().map(|f| {
                    let ident = &f.ident;
                    quote! { ::core::convert::Into::into(self.#ident) }
                });
                quote! {
                    ::core::option::Option::Some(::alloc::boxed::Box::new([
                        #(#exprs),*
                    ]))
                }
            }
            Fields::Unnamed(fields) => {
                let exprs = (0..fields.unnamed.len()).map(|i| {
                    let idx = syn::Index::from(i);
                    quote! { ::core::convert::Into::into(self.#idx) }
                });
                quote! {
                    ::core::option::Option::Some(::alloc::boxed::Box::new([
                        #(#exprs),*
                    ]))
                }
            }
            Fields::Unit => {
                quote! { ::core::option::Option::None }
            }
        };

        quote! {
            ::ember::agent::bdi::literal::Literal::Atom {
                negated: false,
                structure: ::ember::agent::bdi::term::owned::Structure {
                    functor: ::ember::agent::bdi::term::owned::Atom(::alloc::string::String::from(#functor)),
                    arguments: #args,
                }
            }
        }
    }

    fn expand_fields_for_variant(functor: &str, fields: &Fields) -> TokenStream {
        let args = match fields {
            Fields::Named(fields) => {
                let exprs = fields.named.iter().map(|f| {
                    let ident = &f.ident;
                    quote! { ::core::convert::Into::into(#ident) }
                });
                quote! {
                    ::core::option::Option::Some(::alloc::boxed::Box::new([
                        #(#exprs),*
                    ]))
                }
            }
            Fields::Unnamed(fields) => {
                let exprs = (0..fields.unnamed.len()).map(|i| {
                    let ident = syn::Ident::new(&format!("f{}", i), proc_macro2::Span::call_site());
                    quote! { ::core::convert::Into::into(#ident) }
                });
                quote! {
                    ::core::option::Option::Some(::alloc::boxed::Box::new([
                        #(#exprs),*
                    ]))
                }
            }
            Fields::Unit => {
                quote! { ::core::option::Option::None }
            }
        };

        quote! {
            ::ember::agent::bdi::literal::Literal::Atom {
                negated: false,
                structure: ::ember::agent::bdi::term::owned::Structure {
                    functor: ::ember::agent::bdi::term::owned::Atom(::alloc::string::String::from(#functor)),
                    arguments: #args,
                }
            }
        }
    }
}

pub(crate) mod percept {
    //! AI-generated human verified.

    use proc_macro2::TokenStream;
    use quote::quote;
    use syn::DeriveInput;

    pub(crate) fn expand(input: DeriveInput) -> TokenStream {
        let name = &input.ident;
        let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

        quote! {
            impl #impl_generics ::ember::agent::bdi::sensor::Percept for #name #ty_generics #where_clause {
                fn into_beliefs(self) -> impl ::core::iter::IntoIterator<Item = (::ember::agent::bdi::event::Trigger, ::ember::agent::bdi::knowledge::belief::Belief)> {
                    [(
                        ::ember::agent::bdi::event::Trigger::Addition,
                        ::ember::agent::bdi::knowledge::belief::Belief::from(
                            ::ember::agent::bdi::literal::IntoLiteral::into_literal(self)
                        )
                    )]
                }
            }
        }
    }
}
