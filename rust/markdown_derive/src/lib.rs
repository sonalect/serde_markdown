//! Derive macro for `serde_markdown::Markdown`.
//!
//! Apply `#[derive(Markdown)]` to a named struct. Fields marked
//! `#[markdown(body)]` are listed in `BODY_FIELDS` using the Serde field
//! name after `rename`, in declaration order. Types with no body fields
//! still get an empty list.

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::punctuated::Punctuated;
use syn::{
    Data, DataStruct, DeriveInput, Expr, ExprLit, Field, Fields, Lit, LitStr, Meta, Result, Token,
};

/// Implements `serde_markdown::Markdown` for a named struct.
///
/// Body fields are those tagged `#[markdown(body)]`. Each name is the
/// Serde field name after `rename` (serialize name if both serialize and
/// deserialize are given). Other `#[serde(...)]` metas are ignored.
#[proc_macro_derive(Markdown, attributes(markdown))]
pub fn derive_markdown(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand(input: DeriveInput) -> Result<proc_macro2::TokenStream> {
    let fields = named_fields(&input)?;
    let mut body_names = Vec::new();
    for field in fields {
        if is_markdown_body(field)? {
            let name = serde_field_name(field)?;
            body_names.push(LitStr::new(&name, proc_macro2::Span::call_site()));
        }
    }
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics ::serde_markdown::Markdown for #ident #ty_generics #where_clause {
            const BODY_FIELDS: &'static [&'static str] = &[#(#body_names),*];
        }
    })
}

fn named_fields(input: &DeriveInput) -> Result<&Punctuated<Field, Token![,]>> {
    match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(named),
            ..
        }) => Ok(&named.named),
        _ => Err(syn::Error::new_spanned(
            &input.ident,
            "Markdown can only be derived for structs with named fields",
        )),
    }
}

fn is_markdown_body(field: &Field) -> Result<bool> {
    let mut marked = false;
    for attr in &field.attrs {
        if !attr.path().is_ident("markdown") {
            continue;
        }
        let Meta::List(list) = &attr.meta else {
            return Err(expected_body(attr));
        };
        let nested = match list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated) {
            Ok(nested) => nested,
            Err(_) => return Err(expected_body(attr)),
        };
        if nested.is_empty() {
            return Err(expected_body(attr));
        }
        for meta in nested {
            let Meta::Path(path) = &meta else {
                return Err(expected_body(&meta));
            };
            if !path.is_ident("body") {
                return Err(expected_body(path));
            }
            if marked {
                return Err(syn::Error::new_spanned(path, "duplicate `body`"));
            }
            marked = true;
        }
    }
    Ok(marked)
}

fn expected_body(tokens: impl quote::ToTokens) -> syn::Error {
    syn::Error::new_spanned(tokens, "expected `body`")
}

fn serde_field_name(field: &Field) -> Result<String> {
    for attr in &field.attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }
        let Meta::List(list) = &attr.meta else {
            continue;
        };
        let Ok(metas) = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        else {
            continue;
        };
        for meta in metas {
            if let Some(name) = serde_rename_name(&meta) {
                return Ok(name);
            }
        }
    }
    match field.ident.as_ref() {
        Some(ident) => Ok(ident.to_string()),
        None => Err(syn::Error::new_spanned(
            field,
            "Markdown can only be derived for structs with named fields",
        )),
    }
}

fn serde_rename_name(meta: &Meta) -> Option<String> {
    match meta {
        Meta::NameValue(nv) if nv.path.is_ident("rename") => lit_str(&nv.value),
        Meta::List(list) if list.path.is_ident("rename") => {
            let nested = list
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .ok()?;
            let mut serialize = None;
            let mut deserialize = None;
            for item in &nested {
                let Meta::NameValue(nv) = item else {
                    continue;
                };
                let Some(value) = lit_str(&nv.value) else {
                    continue;
                };
                if nv.path.is_ident("serialize") {
                    serialize = Some(value);
                } else if nv.path.is_ident("deserialize") {
                    deserialize = Some(value);
                }
            }
            serialize.or(deserialize)
        }
        _ => None,
    }
}

fn lit_str(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Str(lit), ..
        }) => Some(lit.value()),
        _ => None,
    }
}
