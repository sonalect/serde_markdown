//! Protobuf `example.Page` compiled into this crate and mapped to Markdown.
//!
//! `build.rs` compiles `proto/page.proto` with buffa and injects
//! `#[derive(Markdown)]` plus `#[markdown(body)]` on fields marked
//! `(markdown.body) = true`.

mod generated {
    include!(concat!(env!("OUT_DIR"), "/_include.rs"));
}

/// Generated `example.Page` with Markdown body fields `text1` and `text2`.
pub use generated::example::Page;

/// Canonical fenced YAML document for [`sample`].
pub const EXPECTED_DOCUMENT: &str = "\
```yaml
field1: foo
field2: bar
field3: 1
```
Text1 bla bla bla
---
Text2 bal bla bla";

/// Page used by the binary and tests.
///
/// Proto3 implicit-presence scalars are owned `String` / `i32` (empty / zero
/// means unset). `with_*` setters are emitted only for explicit presence.
#[must_use]
pub fn sample() -> Page {
    Page {
        field1: "foo".into(),
        field2: "bar".into(),
        field3: 1,
        text1: "Text1 bla bla bla".into(),
        text2: "Text2 bal bla bla".into(),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use serde_markdown::{Markdown, from_str, to_string};

    use super::{EXPECTED_DOCUMENT, Page, sample};

    #[test]
    fn to_string_matches_expected_document() {
        assert_eq!(to_string(&sample()).expect("serialize"), EXPECTED_DOCUMENT);
    }

    #[test]
    fn from_str_round_trip_equals_sample() {
        let page = sample();
        let md = to_string(&page).expect("serialize");
        let back: Page = from_str(&md).expect("deserialize");
        assert_eq!(page, back);
    }

    #[test]
    fn body_fields_are_text1_text2() {
        assert_eq!(Page::BODY_FIELDS, ["text1", "text2"]);
    }
}
