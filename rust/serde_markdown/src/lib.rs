//! Serde format crate for Markdown documents with a structured fields fence
//! and ordered body sections.
//!
//! Design: [`DESIGN.md`](https://github.com/sonalect/serde_markdown/blob/main/DESIGN.md).

/// Path of the design document in the repository root.
pub const DESIGN: &str = "DESIGN.md";

mod parse;

#[cfg(test)]
mod testdata;

#[cfg(test)]
mod testdata_smoke {
    use super::testdata::{goldens, types, values};

    #[test]
    fn goldens_are_present() {
        for (name, body) in goldens::ALL {
            assert!(!body.is_empty(), "{name} is empty");
        }
    }

    #[test]
    fn body_field_lists_match_design() {
        assert_eq!(types::Page::BODY_FIELDS, ["text1", "appendix"]);
        assert_eq!(types::FieldsOnly::BODY_FIELDS, [] as [&str; 0]);
        assert_eq!(types::BodyOnly::BODY_FIELDS, ["text1", "text2"]);
        assert_eq!(types::NestedFields::BODY_FIELDS, ["body"]);
        assert_eq!(
            types::OptionalBody::BODY_FIELDS,
            ["first", "middle", "last"]
        );
        assert_eq!(types::JsonNames::BODY_FIELDS, ["bodyNote"]);
        assert_eq!(types::Article::BODY_FIELDS, ["note"]);
        assert_eq!(types::WellKnown::BODY_FIELDS, ["occurredAt", "pause"]);
        assert_eq!(types::Proto3Page::BODY_FIELDS, ["body"]);
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
