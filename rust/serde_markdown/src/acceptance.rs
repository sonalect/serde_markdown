//! Golden walker, complete-value round-trips, and split regressions.

use crate::testdata::goldens;

#[cfg(feature = "yaml")]
use serde::Serialize;
#[cfg(feature = "yaml")]
use serde::de::DeserializeOwned;
#[cfg(feature = "yaml")]
use std::fmt::Debug;

#[cfg(feature = "yaml")]
use crate::markdown::Markdown;
#[cfg(feature = "yaml")]
use crate::testdata::{types, values};
#[cfg(feature = "yaml")]
use crate::{from_str, to_string};

/// Parse-only goldens that are not a `values::*` constructor round-trip.
///
/// - `page.split.fence.md` — inner fenced `---` must not split; the body is
///   not `values::page()`.
/// - `page.split.stars.md` — `***` / `___` must not split; the body is not
///   `values::page()`.
/// - `page.split.list.md` — `---` inside a list item must not split; the body
///   is not `values::page()`.
/// - `page.fence_not_first.md` — first slice is prose; a later `yaml` fence is
///   body, not fields. `from_str` as `Page` / `FieldsOnly` is
///   `ErrorKind::Body`.
const PARSE_ONLY_SKIP: &[&str] = &[
    "page.split.fence.md",
    "page.split.stars.md",
    "page.split.list.md",
    "page.fence_not_first.md",
];

#[derive(Clone, Copy, Debug)]
enum GoldenTarget {
    Page,
    PagePublished,
    FieldsOnly,
    BodyOnly,
    Nested,
    OptionalMiddleNone,
    OptionalTrailingNone,
    OptionalLeadingNone,
    OptionalEmptyString,
    JsonNames,
    Article,
    WellKnown,
    Proto3Page,
    WhitespacePage,
}

fn golden_target(name: &str) -> Option<GoldenTarget> {
    Some(match name {
        "page.fenced.yaml.md"
        | "page.fenced.yml.md"
        | "page.fenced.json.md"
        | "page.fenced.toml.md"
        | "page.bare.yaml.md"
        | "page.bare.json.md"
        | "page.bare.toml.md"
        | "page.unlabeled.yaml.md"
        | "page.unlabeled.json.md"
        | "page.unlabeled.toml.md"
        | "page.leading_prefix.md" => GoldenTarget::Page,
        "page.published.yaml.md" => GoldenTarget::PagePublished,
        "fields_only.fenced.yaml.md" | "fields_only.bare.yaml.md" => GoldenTarget::FieldsOnly,
        "body_only.two_sections.md" => GoldenTarget::BodyOnly,
        "nested.fenced.yaml.md" => GoldenTarget::Nested,
        "optional.middle_none.md" => GoldenTarget::OptionalMiddleNone,
        "optional.trailing_none.md" => GoldenTarget::OptionalTrailingNone,
        "optional.leading_none.md" => GoldenTarget::OptionalLeadingNone,
        "optional.empty_string.md" => GoldenTarget::OptionalEmptyString,
        "names.fenced.yaml.md" => GoldenTarget::JsonNames,
        "structured.fenced.yaml.md" => GoldenTarget::Article,
        "well_known.fenced.yaml.md" => GoldenTarget::WellKnown,
        "proto3.fenced.yaml.md" => GoldenTarget::Proto3Page,
        "whitespace.unicode.md" => GoldenTarget::WhitespacePage,
        _ => return None,
    })
}

#[test]
fn every_golden_is_in_table_or_skip() {
    for skip in PARSE_ONLY_SKIP {
        assert!(
            goldens::ALL.iter().any(|(name, _)| name == skip),
            "parse-only skip {skip} is not in goldens::ALL"
        );
        assert!(
            golden_target(skip).is_none(),
            "parse-only skip {skip} must not also be in the deserialize table"
        );
    }

    for (name, _) in goldens::ALL {
        let skipped = PARSE_ONLY_SKIP.contains(name);
        let handled = golden_target(name).is_some();
        assert!(
            skipped ^ handled,
            "golden {name} must be exactly one of the deserialize table or PARSE_ONLY_SKIP"
        );
    }
}

#[cfg(all(feature = "yaml", feature = "json", feature = "toml"))]
#[test]
fn supported_goldens_deserialize_to_constructors() {
    for (name, body) in goldens::ALL {
        if PARSE_ONLY_SKIP.contains(name) {
            continue;
        }
        let target = golden_target(name).unwrap_or_else(|| {
            panic!("golden {name} is neither deserialized nor in PARSE_ONLY_SKIP")
        });
        match target {
            GoldenTarget::Page => {
                let got: types::Page = from_str(body).unwrap_or_else(|err| panic!("{name}: {err}"));
                assert_eq!(got, values::page(), "{name}");
            }
            GoldenTarget::PagePublished => {
                let got: types::Page = from_str(body).unwrap_or_else(|err| panic!("{name}: {err}"));
                assert_eq!(got, values::page_with_published(), "{name}");
            }
            GoldenTarget::FieldsOnly => {
                let got: types::FieldsOnly =
                    from_str(body).unwrap_or_else(|err| panic!("{name}: {err}"));
                assert_eq!(got, values::fields_only(), "{name}");
            }
            GoldenTarget::BodyOnly => {
                let got: types::BodyOnly =
                    from_str(body).unwrap_or_else(|err| panic!("{name}: {err}"));
                assert_eq!(got, values::body_only(), "{name}");
            }
            GoldenTarget::Nested => {
                let got: types::NestedFields =
                    from_str(body).unwrap_or_else(|err| panic!("{name}: {err}"));
                assert_eq!(got, values::nested(), "{name}");
            }
            GoldenTarget::OptionalMiddleNone => {
                let got: types::OptionalBody =
                    from_str(body).unwrap_or_else(|err| panic!("{name}: {err}"));
                assert_eq!(got, values::optional_middle_none(), "{name}");
            }
            GoldenTarget::OptionalTrailingNone => {
                let got: types::OptionalBody =
                    from_str(body).unwrap_or_else(|err| panic!("{name}: {err}"));
                assert_eq!(got, values::optional_trailing_none(), "{name}");
            }
            GoldenTarget::OptionalLeadingNone => {
                let got: types::OptionalBody =
                    from_str(body).unwrap_or_else(|err| panic!("{name}: {err}"));
                assert_eq!(got, values::optional_leading_none(), "{name}");
            }
            GoldenTarget::OptionalEmptyString => {
                let got: types::OptionalBody =
                    from_str(body).unwrap_or_else(|err| panic!("{name}: {err}"));
                assert_eq!(got, values::optional_empty_string(), "{name}");
            }
            GoldenTarget::JsonNames => {
                let got: types::JsonNames =
                    from_str(body).unwrap_or_else(|err| panic!("{name}: {err}"));
                assert_eq!(got, values::json_names(), "{name}");
            }
            GoldenTarget::Article => {
                let got: types::Article =
                    from_str(body).unwrap_or_else(|err| panic!("{name}: {err}"));
                assert_eq!(got, values::article(), "{name}");
            }
            GoldenTarget::WellKnown => {
                let got: types::WellKnown =
                    from_str(body).unwrap_or_else(|err| panic!("{name}: {err}"));
                assert_eq!(got, values::well_known(), "{name}");
            }
            GoldenTarget::Proto3Page => {
                let got: types::Proto3Page =
                    from_str(body).unwrap_or_else(|err| panic!("{name}: {err}"));
                assert_eq!(got, values::proto3_page(), "{name}");
            }
            GoldenTarget::WhitespacePage => {
                let got: types::Page = from_str(body).unwrap_or_else(|err| panic!("{name}: {err}"));
                assert_eq!(got, values::whitespace_page(), "{name}");
            }
        }
    }
}

#[cfg(feature = "yaml")]
fn assert_round_trip<T>(value: T)
where
    T: Serialize + DeserializeOwned + Markdown + PartialEq + Debug,
{
    let md = to_string(&value).expect("to_string");
    let back: T = from_str(&md).expect("from_str");
    assert_eq!(value, back);
}

/// `from_str(to_string(x)) == x` for complete constructors in `values`.
///
/// Trailing-`None` serialize shapes (`optional_trailing_none` and
/// `optional_trailing_none_generated`) are omitted: those drop the last body
/// section on serialize and are already covered as deserialize goldens.
#[cfg(feature = "yaml")]
#[test]
fn complete_values_round_trip_through_to_string() {
    assert_round_trip(values::page());
    assert_round_trip(values::page_with_published());
    assert_round_trip(values::page_generated());
    assert_round_trip(values::page_generated_with_published());
    assert_round_trip(values::fields_only());
    assert_round_trip(values::fields_only_generated());
    assert_round_trip(values::body_only());
    assert_round_trip(values::body_only_generated());
    assert_round_trip(values::nested());
    assert_round_trip(values::nested_generated());
    assert_round_trip(values::whitespace_page());
    assert_round_trip(values::optional_middle_none());
    assert_round_trip(values::optional_leading_none());
    assert_round_trip(values::optional_empty_string());
    assert_round_trip(values::optional_middle_none_generated());
    assert_round_trip(values::optional_leading_none_generated());
    assert_round_trip(values::optional_empty_string_generated());
    assert_round_trip(values::json_names());
    assert_round_trip(values::json_names_generated());
    assert_round_trip(values::article());
    assert_round_trip(values::article_generated());
    assert_round_trip(values::well_known());
    assert_round_trip(values::well_known_generated());
    assert_round_trip(values::proto3_page());
    assert_round_trip(values::proto3_page_generated());
}

#[cfg(feature = "yaml")]
#[test]
fn page_split_goldens_keep_inner_breaks_in_one_section() {
    let expected = values::page();

    let fence: types::Page = from_str(goldens::PAGE_SPLIT_FENCE).expect("split fence");
    assert_eq!(fence.field1, expected.field1);
    assert_eq!(fence.field2, expected.field2);
    assert_eq!(fence.field3, expected.field3);
    assert_eq!(fence.published, expected.published);
    assert_eq!(fence.appendix, values::TEXT2);
    assert!(
        fence.text1.contains("---"),
        "inner fenced --- must stay in text1: {}",
        fence.text1
    );
    assert!(
        fence.text1.contains("not a split"),
        "inner fence body must stay in text1: {}",
        fence.text1
    );

    let stars: types::Page = from_str(goldens::PAGE_SPLIT_STARS).expect("split stars");
    assert_eq!(stars.field1, expected.field1);
    assert_eq!(stars.field2, expected.field2);
    assert_eq!(stars.field3, expected.field3);
    assert_eq!(stars.published, expected.published);
    assert_eq!(stars.appendix, values::TEXT2);
    assert_eq!(stars.text1, "before\n***\nstill text1\n___\nstill text1");
    assert!(stars.text1.contains("***"));
    assert!(stars.text1.contains("___"));
    assert!(stars.text1.contains("still text1"));

    let list: types::Page = from_str(goldens::PAGE_SPLIT_LIST).expect("split list");
    assert_eq!(list.field1, expected.field1);
    assert_eq!(list.field2, expected.field2);
    assert_eq!(list.field3, expected.field3);
    assert_eq!(list.published, expected.published);
    assert_eq!(list.appendix, values::TEXT2);
    assert_eq!(list.text1, "- keep going\n  ---\n- still the same section");
}
