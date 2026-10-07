//! Derive macro for `serde_markdown::Markdown`.
//!
//! Apply `#[derive(Markdown)]` to a named struct. Fields marked
//! `#[markdown(body)]` are listed in `BODY_FIELDS` using the Serde field
//! name — after the field's `rename` or the struct's `rename_all` — in
//! declaration order. Types with no body fields still get an empty list.

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::punctuated::Punctuated;
use syn::{
    Attribute, Data, DataStruct, DeriveInput, Expr, ExprLit, Field, Fields, Lit, LitStr, Meta,
    Result, Token,
};

/// Implements `serde_markdown::Markdown` for a named struct.
///
/// Body fields are those tagged `#[markdown(body)]`. Each name is the
/// Serde field name: the field's `rename`, else the struct's `rename_all`
/// applied to the field name (a raw identifier loses its `r#`). A body field
/// whose serialize and deserialize names differ is a compile error, since
/// the document could not be read back. Other `#[serde(...)]` metas are
/// ignored.
#[proc_macro_derive(Markdown, attributes(markdown))]
pub fn derive_markdown(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand(input: DeriveInput) -> Result<proc_macro2::TokenStream> {
    let fields = named_fields(&input)?;
    let rename_all = container_rename_all(&input.attrs)?;
    let mut body_names = Vec::new();
    for field in fields {
        if is_markdown_body(field)? {
            let name = serde_field_name(field, &rename_all)?;
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

/// A `rename_all` rule of Serde, applied as Serde applies it to a field
/// name.
#[derive(Clone, Copy)]
enum RenameRule {
    Lower,
    Upper,
    Pascal,
    Camel,
    Snake,
    ScreamingSnake,
    Kebab,
    ScreamingKebab,
}

impl RenameRule {
    fn parse(lit: &LitStr) -> Result<Self> {
        Ok(match lit.value().as_str() {
            "lowercase" => Self::Lower,
            "UPPERCASE" => Self::Upper,
            "PascalCase" => Self::Pascal,
            "camelCase" => Self::Camel,
            "snake_case" => Self::Snake,
            "SCREAMING_SNAKE_CASE" => Self::ScreamingSnake,
            "kebab-case" => Self::Kebab,
            "SCREAMING-KEBAB-CASE" => Self::ScreamingKebab,
            other => {
                return Err(syn::Error::new_spanned(
                    lit,
                    format!("unknown rename rule `{other}`"),
                ));
            }
        })
    }

    fn apply(self, field: &str) -> String {
        match self {
            Self::Lower | Self::Snake => field.to_owned(),
            Self::Upper | Self::ScreamingSnake => field.to_ascii_uppercase(),
            Self::Pascal => {
                let mut pascal = String::with_capacity(field.len());
                let mut capitalize = true;
                for ch in field.chars() {
                    if ch == '_' {
                        capitalize = true;
                    } else if capitalize {
                        pascal.push(ch.to_ascii_uppercase());
                        capitalize = false;
                    } else {
                        pascal.push(ch);
                    }
                }
                pascal
            }
            Self::Camel => {
                let pascal = Self::Pascal.apply(field);
                let mut chars = pascal.chars();
                match chars.next() {
                    Some(first) => first.to_ascii_lowercase().to_string() + chars.as_str(),
                    None => pascal,
                }
            }
            Self::Kebab => field.replace('_', "-"),
            Self::ScreamingKebab => field.to_ascii_uppercase().replace('_', "-"),
        }
    }
}

/// The struct's `rename_all` rules, for serialize and for deserialize.
#[derive(Default)]
struct RenameAll {
    serialize: Option<RenameRule>,
    deserialize: Option<RenameRule>,
}

/// The names a `rename` gives, for serialize and for deserialize.
#[derive(Default)]
struct Renamed {
    serialize: Option<String>,
    deserialize: Option<String>,
}

/// The `#[serde(...)]` metas of `attrs`, in order. Metas that do not parse
/// are left to Serde to report.
fn serde_metas(attrs: &[Attribute]) -> Vec<Meta> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("serde"))
        .filter_map(|attr| match &attr.meta {
            Meta::List(list) => list
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .ok(),
            _ => None,
        })
        .flatten()
        .collect()
}

/// The `serialize` and `deserialize` string values of a list meta such as
/// `rename(serialize = "a", deserialize = "b")`.
fn per_direction(list: &syn::MetaList) -> (Option<LitStr>, Option<LitStr>) {
    let mut serialize = None;
    let mut deserialize = None;
    let Ok(nested) = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated) else {
        return (None, None);
    };
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
    (serialize, deserialize)
}

fn container_rename_all(attrs: &[Attribute]) -> Result<RenameAll> {
    let mut rename_all = RenameAll::default();
    for meta in serde_metas(attrs) {
        match &meta {
            Meta::NameValue(nv) if nv.path.is_ident("rename_all") => {
                if let Some(lit) = lit_str(&nv.value) {
                    let rule = RenameRule::parse(&lit)?;
                    rename_all.serialize = Some(rule);
                    rename_all.deserialize = Some(rule);
                }
            }
            Meta::List(list) if list.path.is_ident("rename_all") => {
                let (serialize, deserialize) = per_direction(list);
                if let Some(lit) = serialize {
                    rename_all.serialize = Some(RenameRule::parse(&lit)?);
                }
                if let Some(lit) = deserialize {
                    rename_all.deserialize = Some(RenameRule::parse(&lit)?);
                }
            }
            _ => {}
        }
    }
    Ok(rename_all)
}

fn field_rename(field: &Field) -> Renamed {
    let mut renamed = Renamed::default();
    for meta in serde_metas(&field.attrs) {
        match &meta {
            Meta::NameValue(nv) if nv.path.is_ident("rename") => {
                if let Some(lit) = lit_str(&nv.value) {
                    renamed.serialize = Some(lit.value());
                    renamed.deserialize = Some(lit.value());
                }
            }
            Meta::List(list) if list.path.is_ident("rename") => {
                let (serialize, deserialize) = per_direction(list);
                if let Some(lit) = serialize {
                    renamed.serialize = Some(lit.value());
                }
                if let Some(lit) = deserialize {
                    renamed.deserialize = Some(lit.value());
                }
            }
            _ => {}
        }
    }
    renamed
}

/// The Serde name of a body field, the same for serialize and deserialize.
fn serde_field_name(field: &Field, rename_all: &RenameAll) -> Result<String> {
    let Some(ident) = field.ident.as_ref() else {
        return Err(syn::Error::new_spanned(
            field,
            "Markdown can only be derived for structs with named fields",
        ));
    };
    let ident = ident.to_string();
    let ident = ident.strip_prefix("r#").unwrap_or(&ident);
    let renamed = field_rename(field);
    let name = |own: Option<String>, rule: Option<RenameRule>| {
        own.unwrap_or_else(|| rule.map_or_else(|| ident.to_owned(), |rule| rule.apply(ident)))
    };
    let serialize = name(renamed.serialize, rename_all.serialize);
    let deserialize = name(renamed.deserialize, rename_all.deserialize);
    if serialize != deserialize {
        return Err(syn::Error::new_spanned(
            field,
            format!(
                "a Markdown body field needs one Serde name; it is `{serialize}` on serialize \
                 and `{deserialize}` on deserialize"
            ),
        ));
    }
    Ok(serialize)
}

fn lit_str(expr: &Expr) -> Option<LitStr> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Str(lit), ..
        }) => Some(lit.clone()),
        _ => None,
    }
}
