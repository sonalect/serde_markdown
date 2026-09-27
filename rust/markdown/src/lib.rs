//! Serde format crate for Markdown documents with a structured fields fence
//! and ordered body sections.
//!
//! A root struct's non-body fields map to an optional first YAML, JSON, or
//! TOML block (fenced or bare). Fields listed in [`Markdown::BODY_FIELDS`]
//! map to body sections split on top-level `---`. Serialize defaults to a
//! labeled `yaml` fence; JSON and TOML only when the caller passes
//! [`Format::Json`] / [`Format::Toml`]. Default crate features are `yaml`,
//! `json`, `toml`, and `derive`. Feature `buffa` is off by default. A fence
//! language not in the compiled features is [`ErrorKind::FormatDisabled`].
//! Documents are UTF-8: [`from_slice`] fails with [`ErrorKind::Syntax`] on
//! invalid bytes. The mapping document path is [`DESIGN`].
//!
//! Google well-known types in the fence or body use that field type's proto3
//! JSON serde. This crate does not parse RFC 3339 itself. Callers who pack
//! protobuf `Any` must install a type registry (`buffa_types::register_wkt_types`)
//! the same way they do for `serde_json`.
//!
//! # Example
//!
//! ```
//! # #[cfg(all(feature = "derive", feature = "yaml"))]
//! # fn main() -> Result<(), serde_markdown::Error> {
//! use serde::{Deserialize, Serialize};
//! use serde_markdown::{from_str, to_string, Markdown};
//!
//! #[derive(Debug, PartialEq, Serialize, Deserialize, Markdown)]
//! struct Page {
//!     title: String,
//!     #[markdown(body)]
//!     body: String,
//! }
//!
//! let page = Page {
//!     title: "Hi".into(),
//!     body: "Hello".into(),
//! };
//! let md = to_string(&page)?;
//! let back: Page = from_str(&md)?;
//! assert_eq!(page, back);
//! # Ok(())
//! # }
//! # #[cfg(not(all(feature = "derive", feature = "yaml")))]
//! # fn main() {}
//! ```

/// Path of the mapping document in the repository root (`DESIGN.md`).
///
/// That file is the format contract: fields fence, body sections, fence
/// languages, features, and error kinds.
pub const DESIGN: &str = "DESIGN.md";

/// Directory containing `markdown/options.proto` for include paths.
///
/// Pass this to buffa or protoc includes so `import "markdown/options.proto"`
/// resolves.
pub const PROTO_INCLUDE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../proto");

#[cfg(feature = "buffa")]
pub mod buffa;

#[cfg(test)]
extern crate self as serde_markdown;

mod de;
mod error;
mod format;
mod markdown;
mod parse;
mod ser;

pub use de::{from_reader, from_slice, from_str};
pub use error::{Error, ErrorKind};
pub use format::{FieldsLayout, Format};
pub use markdown::Markdown;
pub use ser::{to_string, to_string_with, to_string_with_format, to_vec, to_writer};
#[cfg(feature = "derive")]
pub use serde_markdown_derive::Markdown;

#[cfg(test)]
mod testdata;

#[cfg(test)]
mod acceptance;

#[cfg(test)]
mod testdata_smoke {
    use super::Markdown;
    use super::testdata::{goldens, types, values};

    #[test]
    fn goldens_are_present() {
        for (name, body) in goldens::ALL {
            assert!(!body.is_empty(), "{name} is empty");
        }
    }

    #[test]
    fn body_field_lists_match_design() {
        use serde_markdown_proto::markdown::testdata as generated;

        assert_eq!(types::Page::BODY_FIELDS, ["text1", "appendix"]);
        assert_eq!(types::FieldsOnly::BODY_FIELDS, [] as [&str; 0]);
        assert_eq!(types::BodyOnly::BODY_FIELDS, ["text1", "text2"]);
        assert_eq!(types::NestedFields::BODY_FIELDS, ["body"]);
        assert_eq!(
            types::OptionalBody::BODY_FIELDS,
            ["first", "middle", "last"]
        );
        assert_eq!(types::JsonNames::BODY_FIELDS, ["bodyNote"]);
        assert!(
            !types::JsonNames::BODY_FIELDS.contains(&"body_note"),
            "BODY_FIELDS must use the Serde name after rename, not the Rust ident"
        );
        assert!(
            !types::JsonNames::BODY_FIELDS.contains(&"published_at")
                && !types::JsonNames::BODY_FIELDS.contains(&"publishedAt"),
            "front-matter rename must not appear in BODY_FIELDS"
        );
        assert_eq!(types::Article::BODY_FIELDS, ["note"]);
        assert_eq!(types::WellKnown::BODY_FIELDS, ["occurredAt", "pause"]);
        assert_eq!(types::Proto3Page::BODY_FIELDS, ["body"]);

        assert_eq!(generated::Page::BODY_FIELDS, ["text1", "appendix"]);
        assert_eq!(generated::WellKnown::BODY_FIELDS, ["occurredAt", "pause"]);
        assert_eq!(generated::JsonNames::BODY_FIELDS, ["bodyNote"]);
        assert!(
            !generated::JsonNames::BODY_FIELDS.contains(&"body_note"),
            "generated BODY_FIELDS must use the Serde name after rename"
        );
        assert!(
            !generated::JsonNames::BODY_FIELDS.contains(&"published_at")
                && !generated::JsonNames::BODY_FIELDS.contains(&"publishedAt"),
            "generated front-matter rename must not appear in BODY_FIELDS"
        );
        assert_eq!(
            generated::OptionalBody::BODY_FIELDS,
            ["first", "middle", "last"]
        );
        assert_eq!(generated::Article::BODY_FIELDS, ["note"]);
        assert_eq!(generated::Proto3Page::BODY_FIELDS, ["body"]);
    }

    #[test]
    fn constructs_hand_written_and_generated_values() {
        let _ = values::page();
        let _ = values::page_with_published();
        let _ = values::page_generated();
        let _ = values::page_generated_with_published();
        let _ = values::fields_only();
        let _ = values::fields_only_generated();
        let _ = values::body_only();
        let _ = values::body_only_generated();
        let _ = values::nested();
        let _ = values::nested_generated();
        let _ = values::whitespace_page();
        let _ = values::optional_middle_none();
        let _ = values::optional_trailing_none();
        let _ = values::optional_leading_none();
        let _ = values::optional_empty_string();
        let _ = values::optional_middle_none_generated();
        let _ = values::optional_trailing_none_generated();
        let _ = values::optional_leading_none_generated();
        let _ = values::optional_empty_string_generated();
        let _ = values::json_names();
        let _ = values::json_names_generated();
        let _ = values::article();
        let _ = values::article_generated();
        let _ = values::well_known();
        let _ = values::well_known_generated();
        let _ = values::proto3_page();
        let _ = values::proto3_page_generated();
    }

    #[test]
    fn whitespace_golden_keeps_trailing_spaces() {
        assert!(
            goldens::WHITESPACE_UNICODE.contains("spaces   \n"),
            "trailing spaces must stay in the golden file"
        );
    }

    #[cfg(feature = "json")]
    #[test]
    fn proto_json_names_use_camel_case() {
        let value = serde_json::to_value(values::json_names_generated()).expect("json");
        assert!(value.get("publishedAt").is_some());
        assert!(value.get("bodyNote").is_some());
        assert!(value.get("published_at").is_none());
        assert!(value.get("body_note").is_none());
    }
}
