//! Shared resolution for the crate's single helper attribute, `#[ember(...)]`.
//!
//! Every derive that wants configuration owns a snake_case namespace key equal to its own name
//! (e.g. `percept`, `from_term`). Users may write the namespaced form
//! (`#[ember(percept(add(Foo)))]`) to be fully unambiguous when stacking multiple `ember`-aware
//! derives on one item, or the flat form (`#[ember(add(Foo))]`) as shorthand when only one such
//! derive is in play. A derive resolves its own scope by first looking for a nested entry
//! matching its namespace key; if absent, it falls back to the flat top-level entries, picking
//! out only the ones it recognizes and silently ignoring the rest (which belong to some other
//! derive).

use proc_macro2::TokenStream;
use syn::punctuated::Punctuated;
use syn::{Attribute, Meta, Token};

pub(crate) enum Scope<'a> {
    /// A nested entry matching the derive's own namespace key was found; these are its inner
    /// tokens, to be parsed with the derive's own grammar.
    Namespaced(TokenStream),
    /// No nested entry was found; these are the flat top-level entries for the derive to filter
    /// by its own known keys.
    Flat(&'a [Meta]),
}

/// Parse every `#[ember(...)]` attribute on an item into one merged list of `Meta` entries.
pub(crate) fn collect_ember_metas(attrs: &[Attribute]) -> syn::Result<Vec<Meta>> {
    let mut metas = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("ember") {
            let parsed: Punctuated<Meta, Token![,]> =
                attr.parse_args_with(Punctuated::parse_terminated)?;
            metas.extend(parsed);
        }
    }
    Ok(metas)
}

/// Resolve which scope a derive should read its configuration from: a namespaced entry (its own
/// name used as a nested list), or the flat top-level entries as a fallback.
pub(crate) fn resolve_scope<'a>(metas: &'a [Meta], namespace: &str) -> syn::Result<Scope<'a>> {
    let mut found: Option<TokenStream> = None;

    for meta in metas {
        if meta.path().is_ident(namespace) {
            let Meta::List(list) = meta else {
                return Err(syn::Error::new_spanned(
                    meta,
                    format!("`{namespace}` must be used as `{namespace}(...)`"),
                ));
            };

            if found.is_some() {
                return Err(syn::Error::new_spanned(
                    meta,
                    format!("duplicate `{namespace}(...)` entry; only one is allowed per item"),
                ));
            }

            found = Some(list.tokens.clone());
        }
    }

    Ok(match found {
        Some(tokens) => Scope::Namespaced(tokens),
        None => Scope::Flat(metas),
    })
}
