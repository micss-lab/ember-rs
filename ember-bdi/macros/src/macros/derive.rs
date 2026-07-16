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
                                syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site())
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
            #[automatically_derived]
            impl #impl_generics ::ember::agent::bdi::literal::IntoLiteral for #name #ty_generics #where_clause {
                fn into_literal(
                    self,
                ) -> ::ember::agent::bdi::literal::Literal {
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
            ::ember::agent::bdi::literal::Literal {
                negated: false,
                structure: ::ember::agent::bdi::term::owned::Structure {
                    functor: ::ember::agent::bdi::term::owned::Atom(
                        ::alloc::string::String::from(#functor),
                    ),
                    arguments: #args,
                },
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
                    let ident = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
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
            ::ember::agent::bdi::literal::Literal {
                negated: false,
                structure: ::ember::agent::bdi::term::owned::Structure {
                    functor: ::ember::agent::bdi::term::owned::Atom(
                        ::alloc::string::String::from(#functor),
                    ),
                    arguments: #args,
                },
            }
        }
    }
}

pub(crate) mod percept {
    //! `#[derive(Percept)]`, configurable through the shared `#[ember(...)]` helper attribute
    //! (see [`crate::macros::attr`]). Namespace key: `percept`.
    //!
    //! Grammar (flat: `#[ember(add(..))]`; namespaced: `#[ember(percept(add(..)))]`), a
    //! comma-separated list of actions:
    //! - `add` / `add(<expr>)` — emit `(Trigger::Addition, IntoLiteral::into_literal(<expr>))`;
    //!   `<expr>` defaults to `self` when omitted.
    //! - `remove` / `remove(<expr>)` — same, with `Trigger::Deletion`.
    //! - `ignore` — emit nothing for this item/variant (must be alone in its list).
    //!
    //! May be placed on the container (struct, or enum-wide default) and/or on individual enum
    //! variants (a variant's own list overrides the container default; no attribute anywhere
    //! reproduces today's behavior: a single `add(self)`).

    use proc_macro2::TokenStream;
    use quote::quote;
    use syn::parse::Parser;
    use syn::punctuated::Punctuated;
    use syn::{Data, DeriveInput, Field, Fields, Meta, Token, Variant};

    use crate::macros::attr::{self, Scope};

    #[derive(Clone, Copy)]
    enum PerceptActionKind {
        Add,
        Remove,
        Ignore,
    }

    #[derive(Clone)]
    struct PerceptAction {
        kind: PerceptActionKind,
        expr: Option<syn::Expr>,
    }

    impl PerceptAction {
        fn default_add() -> Self {
            PerceptAction {
                kind: PerceptActionKind::Add,
                expr: None,
            }
        }
    }

    /// Convert a list of generic `Meta` entries into `PerceptAction`s. In `strict` mode (used for
    /// the namespaced `percept(...)` scope) every entry must be a recognized action, and errors
    /// out otherwise; in non-strict (flat) mode, entries that don't match a known Percept action
    /// keyword are silently skipped, since they belong to some other derive's configuration.
    fn actions_from_metas(metas: &[Meta], strict: bool) -> syn::Result<Vec<PerceptAction>> {
        let mut actions = Vec::new();

        for meta in metas {
            let kind = match meta.path().get_ident().map(|i| i.to_string()).as_deref() {
                Some("add") => PerceptActionKind::Add,
                Some("remove") => PerceptActionKind::Remove,
                Some("ignore") => PerceptActionKind::Ignore,
                _ if strict => {
                    return Err(syn::Error::new_spanned(
                        meta,
                        "unknown percept action, expected `add`, `add(..)`, `remove`, \
                         `remove(..)`, or `ignore`",
                    ));
                }
                _ => continue,
            };

            let expr = match meta {
                Meta::Path(_) => None,
                Meta::List(list) => {
                    if matches!(kind, PerceptActionKind::Ignore) {
                        return Err(syn::Error::new_spanned(
                            list,
                            "`ignore` does not take arguments",
                        ));
                    }
                    Some(list.parse_args::<syn::Expr>()?)
                }
                Meta::NameValue(nv) => {
                    return Err(syn::Error::new_spanned(
                        nv,
                        "expected `add`, `add(..)`, `remove`, `remove(..)`, or `ignore`",
                    ));
                }
            };

            actions.push(PerceptAction { kind, expr });
        }

        Ok(actions)
    }

    fn validate_actions(actions: &[PerceptAction]) -> syn::Result<()> {
        let has_ignore = actions
            .iter()
            .any(|a| matches!(a.kind, PerceptActionKind::Ignore));

        if has_ignore && actions.len() > 1 {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`ignore` cannot be combined with other actions in the same list",
            ));
        }

        Ok(())
    }

    /// Resolve the effective Percept action list configured directly on `attrs`, if any.
    fn find_percept_actions(attrs: &[syn::Attribute]) -> syn::Result<Option<Vec<PerceptAction>>> {
        let metas = attr::collect_ember_metas(attrs)?;

        let actions = match attr::resolve_scope(&metas, "percept")? {
            Scope::Namespaced(tokens) => {
                let inner: Punctuated<Meta, Token![,]> =
                    Punctuated::parse_terminated.parse2(tokens)?;
                let inner: Vec<Meta> = inner.into_iter().collect();
                let actions = actions_from_metas(&inner, true)?;
                validate_actions(&actions)?;
                Some(actions)
            }
            Scope::Flat(metas) => {
                let actions = actions_from_metas(metas, false)?;
                if actions.is_empty() {
                    None
                } else {
                    validate_actions(&actions)?;
                    Some(actions)
                }
            }
        };

        Ok(actions)
    }

    /// Reject a stray `#[ember(percept(...))]` (or flat Percept-shaped key) placed on a field
    /// instead of the container or enum variant.
    fn check_no_stray_percept_attrs(fields: &Fields) -> syn::Result<()> {
        for field in fields.iter() {
            // `find_percept_actions` always returns `Some(_)` for a namespaced `percept(...)`
            // entry (even an empty one), and `Some(_)` for any recognized flat action key, so
            // this single check catches every stray-on-a-field case.
            if find_percept_actions(&field.attrs)?.is_some() {
                return Err(field_stray_error(field));
            }
        }
        Ok(())
    }

    fn field_stray_error(field: &Field) -> syn::Error {
        syn::Error::new_spanned(
            field,
            "#[ember(...)] Percept configuration is not supported on fields; place it on the \
             struct or enum variant instead",
        )
    }

    fn action_to_tuple(action: &PerceptAction, self_expr: &TokenStream) -> TokenStream {
        let trigger = match action.kind {
            PerceptActionKind::Add => quote!(::ember::agent::bdi::event::Trigger::Addition),
            PerceptActionKind::Remove => quote!(::ember::agent::bdi::event::Trigger::Deletion),
            PerceptActionKind::Ignore => unreachable!("ignore actions are filtered out earlier"),
        };

        let expr = match &action.expr {
            Some(expr) => quote!(#expr),
            None => quote!(#self_expr),
        };

        quote! {
            (#trigger, ::ember::agent::bdi::literal::IntoLiteral::into_literal(#expr))
        }
    }

    fn actions_to_vec_expr(actions: &[PerceptAction], self_expr: &TokenStream) -> TokenStream {
        let tuples: Vec<TokenStream> = actions
            .iter()
            .filter(|a| !matches!(a.kind, PerceptActionKind::Ignore))
            .map(|a| action_to_tuple(a, self_expr))
            .collect();

        if tuples.is_empty() {
            quote! { ::alloc::vec::Vec::new() }
        } else {
            quote! { ::alloc::vec::Vec::from([ #(#tuples),* ]) }
        }
    }

    fn build_enum_arm(variant: &Variant, actions: &[PerceptAction]) -> TokenStream {
        let variant_ident = &variant.ident;

        let self_expr = match &variant.fields {
            Fields::Named(fields) => {
                let idents = fields.named.iter().map(|f| f.ident.as_ref().unwrap());
                quote! { Self::#variant_ident { #(#idents),* } }
            }
            Fields::Unnamed(fields) => {
                let idents = (0..fields.unnamed.len())
                    .map(|i| syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site()));
                quote! { Self::#variant_ident( #(#idents),* ) }
            }
            Fields::Unit => quote! { Self::#variant_ident },
        };

        let pattern = self_expr.clone();
        let vec_expr = actions_to_vec_expr(actions, &self_expr);

        quote! {
            #pattern => #vec_expr,
        }
    }

    pub(crate) fn expand(input: DeriveInput) -> syn::Result<TokenStream> {
        let name = &input.ident;
        let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

        let container_actions = find_percept_actions(&input.attrs)?;

        let body = match &input.data {
            Data::Struct(data) => {
                check_no_stray_percept_attrs(&data.fields)?;
                let actions = container_actions.unwrap_or_else(|| vec![PerceptAction::default_add()]);
                actions_to_vec_expr(&actions, &quote!(self))
            }
            Data::Enum(data) => {
                let mut arms = Vec::with_capacity(data.variants.len());
                for variant in &data.variants {
                    check_no_stray_percept_attrs(&variant.fields)?;
                    let variant_actions = find_percept_actions(&variant.attrs)?;
                    let actions = variant_actions
                        .or_else(|| container_actions.clone())
                        .unwrap_or_else(|| vec![PerceptAction::default_add()]);
                    arms.push(build_enum_arm(variant, &actions));
                }
                quote! {
                    match self {
                        #(#arms)*
                    }
                }
            }
            Data::Union(_) => {
                return Err(syn::Error::new_spanned(
                    name,
                    "Percept cannot be derived for unions",
                ));
            }
        };

        Ok(quote! {
            #[automatically_derived]
            #[allow(unused_variables)]
            impl #impl_generics ::ember::agent::bdi::sensor::Percept for #name #ty_generics #where_clause {
                fn into_beliefs(
                    self,
                ) -> impl ::core::iter::IntoIterator<
                    Item = (
                        ::ember::agent::bdi::event::Trigger,
                        ::ember::agent::bdi::literal::Literal,
                    ),
                > {
                    #body
                }
            }
        })
    }
}

pub(crate) mod from_term {
    //! AI-generated human verified.
    //!
    //! Configurable through the shared `#[ember(...)]` helper attribute (see
    //! [`crate::macros::attr`]). Namespace key: `from_term`. Only flag today: `transparent`,
    //! either flat (`#[ember(transparent)]`) or namespaced (`#[ember(from_term(transparent))]`).

    use heck::ToSnakeCase;
    use proc_macro2::TokenStream;
    use quote::quote;
    use syn::parse::Parser;
    use syn::punctuated::Punctuated;
    use syn::{Data, DeriveInput, Fields, Meta, Token};

    use crate::macros::attr::{self, Scope};

    fn is_transparent(attrs: &[syn::Attribute]) -> syn::Result<bool> {
        let metas = attr::collect_ember_metas(attrs)?;

        let is_transparent = match attr::resolve_scope(&metas, "from_term")? {
            Scope::Namespaced(tokens) => {
                let inner: Punctuated<Meta, Token![,]> =
                    Punctuated::parse_terminated.parse2(tokens)?;
                inner.iter().any(|m| m.path().is_ident("transparent"))
            }
            Scope::Flat(metas) => metas.iter().any(|m| m.path().is_ident("transparent")),
        };

        Ok(is_transparent)
    }

    pub(crate) fn expand(input: DeriveInput) -> syn::Result<TokenStream> {
        let name = &input.ident;
        let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

        let is_transparent = is_transparent(&input.attrs)?;

        let body = match &input.data {
            Data::Struct(data) => {
                if is_transparent {
                    match &data.fields {
                        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                            let inner_ty = &fields.unnamed[0].ty;
                            quote! {
                                Ok(Self(
                                    <#inner_ty as ::ember::agent::bdi::term::FromTerm>::from_term(
                                        term,
                                    )?,
                                ))
                            }
                        }
                        _ => {
                            return Err(syn::Error::new_spanned(
                                name,
                                "#[ember(transparent)] is only supported on single-element tuple structs",
                            ));
                        }
                    }
                } else {
                    let functor = name.to_string().to_snake_case();
                    expand_fields_for_struct(name, &functor, &data.fields)
                }
            }
            Data::Enum(data) => {
                if is_transparent {
                    return Err(syn::Error::new_spanned(
                        name,
                        "#[ember(transparent)] cannot be used on enums",
                    ));
                }

                let match_arms = data.variants.iter().map(|v| {
                    let variant_name = &v.ident;
                    let functor = variant_name.to_string().to_snake_case();
                    expand_fields_for_enum_variant(variant_name, &functor, &v.fields)
                });

                quote! {
                    match term {
                        ::ember::agent::bdi::term::reference::TermRef::Literal {
                            functor,
                            arguments,
                            ..
                        } => match functor.0.as_str() {
                            #(#match_arms,)*
                            _ => Err(
                                ::ember::agent::bdi::term::FromTermError::InvalidType(
                                    ::core::option::Option::Some(::core::stringify!(#name)),
                                ),
                            ),
                        },
                        _ => Err(
                            ::ember::agent::bdi::term::FromTermError::InvalidType(
                                ::core::option::Option::Some(::core::stringify!(#name)),
                            ),
                        ),
                    }
                }
            }
            Data::Union(_) => {
                return Err(syn::Error::new_spanned(
                    name,
                    "FromTerm cannot be derived for unions",
                ));
            }
        };

        Ok(quote! {
            #[automatically_derived]
            impl #impl_generics ::ember::agent::bdi::term::FromTerm<'_> for #name #ty_generics #where_clause {
                fn from_term(
                    term: ::ember::agent::bdi::term::reference::TermRef<'_>,
                ) -> ::core::result::Result<Self, ::ember::agent::bdi::term::FromTermError> {
                    #body
                }
            }
        })
    }

    fn expand_fields_for_struct(name: &syn::Ident, functor: &str, fields: &Fields) -> TokenStream {
        match fields {
            Fields::Named(fields) => {
                let arg_count = fields.named.len();
                let args_extraction = fields.named.iter().enumerate().map(|(i, f)| {
                    let ident = &f.ident;
                    let ty = &f.ty;
                    quote! {
                        #ident: <#ty as ::ember::agent::bdi::term::FromTerm>::from_term(
                            (&arguments[#i]).clone(),
                        )?
                    }
                });
                quote! {
                    match term {
                        ::ember::agent::bdi::term::reference::TermRef::Literal {
                            functor: term_functor,
                            arguments,
                            ..
                        } if term_functor.0 == #functor && arguments.len() == #arg_count => {
                            Ok(Self {
                                #(#args_extraction,)*
                            })
                        }
                        _ => Err(
                            ::ember::agent::bdi::term::FromTermError::InvalidType(
                                ::core::option::Option::Some(::core::stringify!(#name)),
                            ),
                        ),
                    }
                }
            }
            Fields::Unnamed(fields) => {
                let arg_count = fields.unnamed.len();
                let args_extraction = fields.unnamed.iter().enumerate().map(|(i, f)| {
                    let ty = &f.ty;
                    quote! {
                        <#ty as ::ember::agent::bdi::term::FromTerm>::from_term(
                            (&arguments[#i]).clone(),
                        )?
                    }
                });
                quote! {
                    match term {
                        ::ember::agent::bdi::term::reference::TermRef::Literal {
                            functor: term_functor,
                            arguments,
                            ..
                        } if term_functor.0 == #functor && arguments.len() == #arg_count => {
                            Ok(Self (
                                #(#args_extraction,)*
                            ))
                        }
                        _ => Err(
                            ::ember::agent::bdi::term::FromTermError::InvalidType(
                                ::core::option::Option::Some(::core::stringify!(#name)),
                            ),
                        ),
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    match term {
                        ::ember::agent::bdi::term::reference::TermRef::Literal {
                            functor: term_functor,
                            arguments,
                            ..
                        } if term_functor.0 == #functor && arguments.is_empty() => {
                            Ok(Self)
                        }
                        _ => Err(
                            ::ember::agent::bdi::term::FromTermError::InvalidType(
                                ::core::option::Option::Some(::core::stringify!(#name)),
                            ),
                        ),
                    }
                }
            }
        }
    }

    fn expand_fields_for_enum_variant(
        v_name: &syn::Ident,
        expected_functor: &str,
        fields: &Fields,
    ) -> TokenStream {
        match fields {
            Fields::Named(fields) => {
                let arg_count = fields.named.len();
                let args_extraction = fields.named.iter().enumerate().map(|(i, f)| {
                    let ident = &f.ident;
                    let ty = &f.ty;
                    quote! {
                        #ident: <#ty as ::ember::agent::bdi::term::FromTerm>::from_term(
                            (&arguments[#i]).clone(),
                        )?
                    }
                });
                quote! {
                    #expected_functor if arguments.len() == #arg_count => {
                        Ok(Self::#v_name {
                            #(#args_extraction,)*
                        })
                    }
                }
            }
            Fields::Unnamed(fields) => {
                let arg_count = fields.unnamed.len();
                let args_extraction = fields.unnamed.iter().enumerate().map(|(i, f)| {
                    let ty = &f.ty;
                    quote! {
                        <#ty as ::ember::agent::bdi::term::FromTerm>::from_term(
                            (&arguments[#i]).clone(),
                        )?
                    }
                });
                quote! {
                    #expected_functor if arguments.len() == #arg_count => {
                        Ok(Self::#v_name (
                            #(#args_extraction,)*
                        ))
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    #expected_functor if arguments.is_empty() => {
                        Ok(Self::#v_name)
                    }
                }
            }
        }
    }
}
