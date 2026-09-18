//! Hand-written `Page` mapped to a fenced YAML Markdown document.

use serde::{Deserialize, Serialize};
use serde_markdown::Markdown;

/// Sample document: labeled `yaml` fence and two body sections.
#[derive(Debug, PartialEq, Serialize, Deserialize, Markdown)]
pub struct Page {
    pub field1: String,
    pub field2: String,
    pub field3: i32,
    #[markdown(body)]
    pub text1: String,
    #[markdown(body)]
    pub text2: String,
}

/// Canonical fenced YAML document for [`sample`].
pub const EXPECTED_DOCUMENT: &str = "\
```yaml
field1: foo
field2: bar
field3: 1
```
Text1 bla bla bla
---
Text2 bal bla bla
";

/// Page used by the binary and tests.
#[must_use]
pub fn sample() -> Page {
    Page {
        field1: "foo".into(),
        field2: "bar".into(),
        field3: 1,
        text1: "Text1 bla bla bla".into(),
        text2: "Text2 bal bla bla".into(),
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
