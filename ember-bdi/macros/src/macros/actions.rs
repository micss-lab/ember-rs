//! Generate an action enum and factory methods to construct actions based on the user-defined
//! bdi-actions.
//!
//! AI-generated, human verified, implementation.

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{Error, FnArg, ImplItem, ImplItemFn, ItemImpl, Lit, Meta, Pat, Token, Type, TypePath};

pub(crate) fn expand(mut input: ItemImpl) -> syn::Result<TokenStream> {
    let self_ty_ident = extract_impl_ident(&input.self_ty)?;
    let action_enum_ident = format_ident!("{}Action", self_ty_ident);

    let mut generated = GeneratedActions::default();

    let items = std::mem::take(&mut input.items);
    for item in items {
        if let ImplItem::Fn(method) = item {
            generated.process_method(method, &action_enum_ident, &mut input.items)?;
        } else {
            input.items.push(item);
        }
    }

    for fm in &generated.factory_methods {
        let parsed_fm: ImplItem = syn::parse2(fm.clone())?;
        input.items.push(parsed_fm);
    }

    let variants = &generated.variants;
    let match_arms = &generated.execute_match_arms;
    let self_ty = &input.self_ty;

    Ok(quote! {
        #input

        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
        pub(crate) enum #action_enum_ident {
            #(#variants),*
        }

        impl ::ember::agent::bdi::plan::action::Execute for #action_enum_ident {
            type State = #self_ty;
            type Action = Self;

            fn execute(
                self,
                bindings: &impl ::ember::agent::bdi::bindings::BindingLookup,
                context: &mut ::ember::agent::bdi::context::Context<Self::Action>,
                state: &mut Self::State,
            ) -> Option<Self> {
                match self {
                    #(#match_arms)*
                }
            }
        }
    })
}

fn extract_impl_ident(self_ty: &Type) -> syn::Result<Ident> {
    if let Type::Path(TypePath { path, .. }) = self_ty {
        if let Some(segment) = path.segments.last() {
            return Ok(segment.ident.clone());
        }
    }
    Err(Error::new(
        syn::spanned::Spanned::span(self_ty),
        "Unsupported impl type",
    ))
}

#[derive(Default)]
struct GeneratedActions {
    variants: Vec<TokenStream>,
    execute_match_arms: Vec<TokenStream>,
    factory_methods: Vec<TokenStream>,
}

impl GeneratedActions {
    fn process_method(
        &mut self,
        mut method: ImplItemFn,
        action_enum_ident: &Ident,
        new_items: &mut Vec<ImplItem>,
    ) -> syn::Result<()> {
        use heck::ToPascalCase;

        let action_name = extract_bdi_action_name(&mut method.attrs)?;
        let method_ident = &method.sig.ident;
        let method_name_str = action_name.unwrap_or_else(|| method_ident.to_string());

        let variant_ident = Ident::new(&method_name_str.to_pascal_case(), method.sig.ident.span());
        let factory_ident = format_ident!("{}_action", method_name_str);

        let ActionParams {
            variant_fields,
            factory_params,
            variant_field_names,
            execute_resolutions,
            method_call_args,
        } = extract_action_params(&method.sig.inputs);

        self.factory_methods.push(quote! {
            pub(crate) fn #factory_ident(#(#factory_params),*) -> #action_enum_ident {
                #action_enum_ident::#variant_ident {
                    #(#variant_field_names),*
                }
            }
        });

        self.variants.push(quote! {
            #variant_ident {
                #(#variant_fields),*
            }
        });

        self.execute_match_arms.push(quote! {
            #action_enum_ident::#variant_ident { #(#variant_field_names),* } => {
                #(#execute_resolutions)*
                state.#method_ident(#(#method_call_args),*);
                None
            }
        });

        new_items.push(ImplItem::Fn(method));
        Ok(())
    }
}

fn extract_bdi_action_name(attrs: &mut Vec<syn::Attribute>) -> syn::Result<Option<String>> {
    let mut custom_name = None;
    let mut remaining_attrs = Vec::new();

    for attr in attrs.drain(..) {
        if attr.path().is_ident("bdi_action") {
            let nested = attr.parse_args_with(
                syn::punctuated::Punctuated::<Meta, Token![,]>::parse_terminated,
            )?;
            for meta in nested {
                if let Meta::NameValue(nv) = meta {
                    if nv.path.is_ident("name") {
                        if let syn::Expr::Lit(expr_lit) = &nv.value {
                            if let Lit::Str(lit_str) = &expr_lit.lit {
                                custom_name = Some(lit_str.value());
                            }
                        }
                    }
                }
            }
        } else {
            remaining_attrs.push(attr);
        }
    }

    *attrs = remaining_attrs;
    Ok(custom_name)
}

#[derive(Default)]
struct ActionParams {
    variant_fields: Vec<TokenStream>,
    factory_params: Vec<TokenStream>,
    variant_field_names: Vec<Ident>,
    execute_resolutions: Vec<TokenStream>,
    method_call_args: Vec<TokenStream>,
}

fn extract_action_params(inputs: &syn::punctuated::Punctuated<FnArg, Token![,]>) -> ActionParams {
    let mut params = ActionParams::default();

    for arg in inputs {
        if let FnArg::Typed(pat_type) = arg {
            let is_context = is_context_type(&pat_type.ty);

            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                let param_ident = &pat_ident.ident;
                let clean_name = param_ident.to_string().trim_start_matches('_').to_string();
                let clean_ident = Ident::new(&clean_name, param_ident.span());

                if is_context {
                    params.method_call_args.push(quote! { context });
                } else {
                    let ty = &pat_type.ty;
                    params
                        .variant_fields
                        .push(quote! { #clean_ident: ::ember::agent::bdi::term::owned::Term });
                    params
                        .factory_params
                        .push(quote! { #clean_ident: ::ember::agent::bdi::term::owned::Term });
                    params.variant_field_names.push(clean_ident.clone());

                    let err_msg_res = format!("Failed to resolve argument '{clean_ident}'");
                    let err_msg_conv =
                        format!("Failed to convert argument '{clean_ident}' to expected type");

                    params.execute_resolutions.push(quote! {
                        let #clean_ident = match ::ember::agent::bdi::resolve::Resolve::resolve_as_view(&#clean_ident, bindings) {
                            Ok(val) => val,
                            Err(e) => {
                                ::log::error!("{}: {:?}", #err_msg_res, e);
                                return None;
                            }
                        };
                        let #clean_ident = match <#ty as ::ember::agent::bdi::term::conversion::FromTerm>::from_term(#clean_ident.into()) {
                            Ok(val) => val,
                            Err(e) => {
                                ::log::error!("{}: {e}", #err_msg_conv);
                                return None;
                            }
                        };
                    });

                    params.method_call_args.push(quote! { #clean_ident });
                }
            }
        }
    }

    params
}

fn is_context_type(ty: &Type) -> bool {
    let path = match ty {
        Type::Reference(r) => match &*r.elem {
            Type::Path(p) => &p.path,
            _ => return false,
        },
        Type::Path(p) => &p.path,
        _ => return false,
    };

    path.segments.last().is_some_and(|s| s.ident == "Context")
}
