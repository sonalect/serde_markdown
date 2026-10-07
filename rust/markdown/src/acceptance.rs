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

/// Texts the last body field must carry byte for byte: top-level `---`
/// lines, a GFM table, a setext underline, a fence with `---` inside, the
/// two characters `""`, empty text, leading and trailing line breaks, CRLF,
/// Cyrillic, and text that looks like front matter.
#[cfg(feature = "yaml")]
const HARD_TEXTS: &[&str] = &[
    "---",
    "a\n---\nb",
    "---\nstarts with a rule",
    "ends with a rule\n---",
    "ends with a rule and a break\n---\n",
    "| a | b |\n|---|---|\n| 1 | 2 |\n",
    "Title\n---\n\nParagraph",
    "```\n---\n```\n",
    "```yaml\nkey: value\n```\n",
    "\"\"",
    "",
    "\n",
    "\n\n",
    "\nleading break",
    "trailing break\n",
    "two trailing breaks\n\n",
    "   indented start",
    "line one\r\nline two\r\n",
    "Кириллица — текст.\nВторая строка\n",
    "key: value\nother: 1\n",
    "***\n___\n",
];

/// Round-trip `value` through both fields layouts in YAML.
#[cfg(feature = "yaml")]
fn assert_round_trip_both_layouts<T>(value: &T, what: &str)
where
    T: Serialize + DeserializeOwned + Markdown + PartialEq + Debug,
{
    use crate::{FieldsLayout, Format, to_string_with};

    for layout in [FieldsLayout::Fenced, FieldsLayout::Bare] {
        let md = to_string_with(value, Format::Yaml, layout)
            .unwrap_or_else(|err| panic!("{what} {layout:?}: {err}"));
        let back: T = from_str(&md).unwrap_or_else(|err| panic!("{what} {layout:?}: {err}\n{md}"));
        assert_eq!(&back, value, "{what} {layout:?}, document:\n{md}");
    }
}

#[cfg(feature = "yaml")]
#[test]
fn the_last_body_field_round_trips_every_hard_text_in_both_layouts() {
    for text in HARD_TEXTS {
        let hand = types::Proto3Page {
            title: "t".into(),
            body: Some((*text).to_owned()),
        };
        assert_round_trip_both_layouts(&hand, &format!("single {text:?}"));
        let generated = values::proto3_page_generated().with_body(*text);
        assert_round_trip_both_layouts(&generated, &format!("generated {text:?}"));
        let mut page = values::page();
        page.appendix = (*text).to_owned();
        assert_round_trip_both_layouts(&page, &format!("last of two {text:?}"));
        let three = types::OptionalBody {
            title: "memo".into(),
            first: Some("a".into()),
            middle: None,
            last: Some((*text).to_owned()),
        };
        assert_round_trip_both_layouts(&three, &format!("last of three {text:?}"));
    }
    for (first, middle, last) in [
        (None, None, Some("")),
        (Some(""), None, None),
        (None, Some(""), None),
        (Some(""), Some(""), Some("")),
        (None, None, None),
        (Some("a"), None, None),
    ] {
        let value = types::OptionalBody {
            title: "memo".into(),
            first: first.map(str::to_owned),
            middle: middle.map(str::to_owned),
            last: last.map(str::to_owned),
        };
        assert_round_trip_both_layouts(&value, "presence matrix");
    }
    for (text1, text2) in [("", ""), ("", "x"), ("x", ""), ("\n", "---")] {
        let value = types::BodyOnly {
            text1: text1.into(),
            text2: text2.into(),
        };
        assert_round_trip_both_layouts(&value, "body only");
    }
}

#[cfg(feature = "yaml")]
#[test]
fn the_last_body_field_round_trips_every_hard_text_verbatim() {
    for text in HARD_TEXTS {
        let hand = types::Proto3Page {
            title: "t".into(),
            body: Some((*text).to_owned()),
        };
        let md = to_string(&hand).unwrap_or_else(|err| panic!("{text:?}: {err}"));
        let back: types::Proto3Page =
            from_str(&md).unwrap_or_else(|err| panic!("{text:?}: {err}\n{md}"));
        assert_eq!(back, hand, "hand-written, document:\n{md}");

        let generated = values::proto3_page_generated().with_body(*text);
        let md = to_string(&generated).unwrap_or_else(|err| panic!("{text:?}: {err}"));
        let back = from_str(&md).unwrap_or_else(|err| panic!("{text:?}: {err}\n{md}"));
        assert_eq!(generated, back, "generated, document:\n{md}");

        let mut page = values::page();
        page.appendix = (*text).to_owned();
        let md = to_string(&page).unwrap_or_else(|err| panic!("{text:?}: {err}"));
        let back: types::Page = from_str(&md).unwrap_or_else(|err| panic!("{text:?}: {err}\n{md}"));
        assert_eq!(back, page, "last of two body fields, document:\n{md}");

        let three = types::OptionalBody {
            title: "memo".into(),
            first: Some("a".into()),
            middle: None,
            last: Some((*text).to_owned()),
        };
        let md = to_string(&three).unwrap_or_else(|err| panic!("{text:?}: {err}"));
        let back: types::OptionalBody =
            from_str(&md).unwrap_or_else(|err| panic!("{text:?}: {err}\n{md}"));
        assert_eq!(back, three, "last of three body fields, document:\n{md}");
    }
}

#[cfg(feature = "yaml")]
#[test]
fn the_document_ends_with_the_last_byte_of_the_last_body_field() {
    let page = types::Proto3Page {
        title: "t".into(),
        body: Some("no break at the end".into()),
    };
    assert!(
        to_string(&page)
            .expect("serialize")
            .ends_with("```\nno break at the end")
    );
    let page = types::Proto3Page {
        title: "t".into(),
        body: Some("one break\n".into()),
    };
    assert!(
        to_string(&page)
            .expect("serialize")
            .ends_with("```\none break\n")
    );
}

#[cfg(feature = "yaml")]
#[test]
fn an_absent_an_empty_and_a_quoted_last_field_are_three_documents() {
    let shapes = [None, Some(String::new()), Some("\"\"".to_owned())];
    let documents: Vec<String> = shapes
        .iter()
        .map(|body| {
            let page = types::Proto3Page {
                title: "t".into(),
                body: body.clone(),
            };
            assert_round_trip_both_layouts(&page, "presence");
            to_string(&page).expect("serialize")
        })
        .collect();
    assert_ne!(documents[0], documents[1]);
    assert_ne!(documents[1], documents[2]);
    assert_ne!(documents[0], documents[2]);
}

#[cfg(feature = "yaml")]
#[test]
fn a_middle_body_value_with_a_top_level_rule_is_refused_on_write() {
    let mut page = values::page();
    page.text1 = "before\n---\nafter".into();
    let err = to_string(&page).expect_err("middle value with ---");
    assert_eq!(err.kind(), crate::ErrorKind::Body);

    let three = types::OptionalBody {
        title: "memo".into(),
        first: Some("x\n---\ny".into()),
        middle: None,
        last: Some("z".into()),
    };
    let err = to_string(&three).expect_err("middle value with ---");
    assert_eq!(err.kind(), crate::ErrorKind::Body);

    let mut fenced = values::page();
    fenced.text1 = "```\n---\n```".into();
    let md = to_string(&fenced).expect("a fenced --- does not split");
    let back: types::Page = from_str(&md).expect("deserialize");
    assert_eq!(back, fenced);
}

#[cfg(feature = "yaml")]
#[test]
fn a_body_only_document_keeps_a_leading_break_and_a_leading_rule() {
    for (text1, text2) in [
        ("\n\nstarts with breaks", "last"),
        ("   starts with spaces", "---\nlast starts with a rule"),
        ("key: value", "looks like fields before it"),
    ] {
        let value = types::BodyOnly {
            text1: text1.into(),
            text2: text2.into(),
        };
        let md = to_string(&value).expect("serialize");
        let back: types::BodyOnly = from_str(&md).expect("deserialize");
        assert_eq!(back, value, "document:\n{md}");
    }
}

/// Fields written in declaration order, not alphabetically.
#[cfg(feature = "yaml")]
#[derive(
    Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, serde_markdown_derive::Markdown,
)]
struct Ordered {
    zeta: String,
    alpha: i32,
    mid: bool,
    #[markdown(body)]
    body: String,
}

#[cfg(all(feature = "yaml", feature = "json", feature = "toml"))]
#[test]
fn front_matter_keys_follow_declaration_order_in_every_format() {
    use crate::{FieldsLayout, Format, to_string_with};

    let value = Ordered {
        zeta: "z".into(),
        alpha: 1,
        mid: true,
        body: "text".into(),
    };
    for format in [Format::Yaml, Format::Json, Format::Toml] {
        for layout in [FieldsLayout::Fenced, FieldsLayout::Bare] {
            let md = to_string_with(&value, format, layout).expect("serialize");
            let zeta = md.find("zeta").expect("zeta");
            let alpha = md.find("alpha").expect("alpha");
            let mid = md.find("mid").expect("mid");
            assert!(zeta < alpha && alpha < mid, "{format} {layout:?}:\n{md}");
            let back: Ordered = from_str(&md).expect("deserialize");
            assert_eq!(back, value);
        }
    }

    let generated = values::well_known_generated();
    let md = to_string(&generated).expect("serialize");
    let keys: Vec<&str> = md
        .lines()
        .skip(1)
        .take_while(|line| !line.starts_with("```"))
        .filter(|line| !line.starts_with(' '))
        .filter_map(|line| line.split(':').next())
        .collect();
    assert_eq!(
        keys,
        ["published", "ttl", "meta", "extra", "mask", "count"],
        "{md}"
    );
}

#[cfg(feature = "yaml")]
#[test]
fn front_matter_keys_are_read_in_any_order() {
    let md = "```yaml\nmid: true\nalpha: 1\nzeta: z\n```\ntext";
    let back: Ordered = from_str(md).expect("deserialize");
    assert_eq!(
        back,
        Ordered {
            zeta: "z".into(),
            alpha: 1,
            mid: true,
            body: "text".into(),
        }
    );
}

/// Regressions from the full review: enums, `rename_all`, byte bodies that
/// read as lists, `#[serde(default)]` body fields, TOML date-times.
#[cfg(all(feature = "yaml", feature = "json", feature = "toml"))]
mod review {
    use serde::{Deserialize, Serialize};

    use crate::{ErrorKind, FieldsLayout, Format, Markdown, from_str, to_string, to_string_with};

    const LANGUAGES: [Format; 3] = [Format::Yaml, Format::Json, Format::Toml];
    const LAYOUTS: [FieldsLayout; 2] = [FieldsLayout::Fenced, FieldsLayout::Bare];

    fn round_trip<T>(value: &T, formats: &[Format], what: &str)
    where
        T: Serialize + serde::de::DeserializeOwned + Markdown + PartialEq + std::fmt::Debug,
    {
        for &format in formats {
            for layout in LAYOUTS {
                let md = to_string_with(value, format, layout)
                    .unwrap_or_else(|err| panic!("{what} {format} {layout:?}: {err}"));
                let back: T = from_str(&md)
                    .unwrap_or_else(|err| panic!("{what} {format} {layout:?}: {err}\n{md}"));
                assert_eq!(&back, value, "{what} {format} {layout:?}:\n{md}");
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    enum Status {
        Draft,
        #[serde(rename = "live")]
        Published,
        Moved(u32),
        Pair(i32, i32),
        Point {
            x: i32,
            y: i32,
        },
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, serde_markdown_derive::Markdown)]
    struct Tagged {
        status: Status,
        history: Vec<Status>,
        #[markdown(body)]
        mood: Status,
        #[markdown(body)]
        maybe: Option<Status>,
        #[markdown(body)]
        text: String,
    }

    #[test]
    fn enums_round_trip_in_the_fields_and_in_the_body() {
        let tagged = Tagged {
            status: Status::Published,
            history: vec![
                Status::Draft,
                Status::Moved(3),
                Status::Pair(1, 2),
                Status::Point { x: 1, y: 2 },
            ],
            mood: Status::Draft,
            maybe: Some(Status::Published),
            text: "body".into(),
        };
        round_trip(&tagged, &LANGUAGES, "unit variants in the body");
        let structured = Tagged {
            mood: Status::Point { x: 4, y: 5 },
            maybe: Some(Status::Moved(7)),
            ..tagged.clone()
        };
        round_trip(&structured, &LANGUAGES, "variants with content in the body");
        let absent = Tagged {
            maybe: None,
            ..tagged
        };
        round_trip(&absent, &LANGUAGES, "an absent enum body field");
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, serde_markdown_derive::Markdown)]
    #[serde(rename_all = "camelCase")]
    struct Camel {
        page_title: String,
        #[markdown(body)]
        body_note: String,
        #[serde(rename = "kept")]
        #[markdown(body)]
        renamed_field: String,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, serde_markdown_derive::Markdown)]
    #[serde(rename_all(serialize = "kebab-case", deserialize = "kebab-case"))]
    struct Kebab {
        #[markdown(body)]
        long_name: String,
        #[markdown(body)]
        r#type: String,
    }

    #[derive(Serialize, serde_markdown_derive::Markdown)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    struct Screaming {
        #[markdown(body)]
        some_field: String,
    }

    #[derive(Serialize, serde_markdown_derive::Markdown)]
    #[serde(rename_all = "PascalCase")]
    struct Pascal {
        #[markdown(body)]
        some_field: String,
    }

    #[derive(Serialize, serde_markdown_derive::Markdown)]
    #[serde(rename_all = "SCREAMING-KEBAB-CASE")]
    struct ScreamingKebab {
        #[markdown(body)]
        some_field: String,
    }

    #[test]
    fn body_names_follow_rename_all() {
        assert_eq!(Camel::BODY_FIELDS, ["bodyNote", "kept"]);
        assert_eq!(Kebab::BODY_FIELDS, ["long-name", "type"]);
        assert_eq!(Screaming::BODY_FIELDS, ["SOME_FIELD"]);
        assert_eq!(Pascal::BODY_FIELDS, ["SomeField"]);
        assert_eq!(ScreamingKebab::BODY_FIELDS, ["SOME-FIELD"]);

        let camel = Camel {
            page_title: "t".into(),
            body_note: "the note".into(),
            renamed_field: "the rest".into(),
        };
        let md = to_string(&camel).expect("serialize");
        assert_eq!(md, "```yaml\npageTitle: t\n```\nthe note\n---\nthe rest");
        round_trip(&camel, &LANGUAGES, "rename_all camelCase");
        let kebab = Kebab {
            long_name: "a".into(),
            r#type: "b".into(),
        };
        round_trip(
            &kebab,
            &LANGUAGES,
            "rename_all kebab-case and a raw identifier",
        );
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, serde_markdown_derive::Markdown)]
    struct Bytes {
        title: String,
        #[markdown(body)]
        data: Vec<u8>,
        #[markdown(body)]
        tags: Vec<String>,
    }

    #[test]
    fn a_byte_body_that_reads_as_a_list_stays_bytes() {
        for data in [
            b"[1, 2]".as_slice(),
            b"- a\n- b",
            b"{\"k\": 1}",
            b"plain text",
            b"",
        ] {
            let value = Bytes {
                title: "t".into(),
                data: data.to_vec(),
                tags: vec!["x".into(), "y".into()],
            };
            round_trip(&value, &[Format::Yaml, Format::Json], &format!("{data:?}"));
        }
        let ambiguous = Bytes {
            title: "t".into(),
            data: b"[]".to_vec(),
            tags: Vec::new(),
        };
        let err = to_string(&ambiguous).expect_err("bytes that read as an empty list");
        assert_eq!(err.kind(), ErrorKind::Body);
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, serde_markdown_derive::Markdown)]
    struct Defaults {
        title: String,
        #[markdown(body)]
        first: String,
        #[serde(default)]
        #[markdown(body)]
        notes: String,
    }

    #[test]
    fn a_body_field_with_a_serde_default_may_be_missing() {
        let back: Defaults = from_str("```yaml\ntitle: t\n```\nonly the first").expect("default");
        assert_eq!(back.first, "only the first");
        assert_eq!(back.notes, "");
        let err = from_str::<Defaults>("```yaml\ntitle: t\n```\n").expect_err("first is required");
        assert_eq!(err.kind(), ErrorKind::Body);
        assert!(err.to_string().contains("first"), "{err}");
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, serde_markdown_derive::Markdown)]
    struct When {
        when: String,
        at: buffa_types::google::protobuf::Timestamp,
        #[markdown(body)]
        text: String,
    }

    #[test]
    fn a_toml_date_time_reads_as_its_text() {
        let md = "```toml\nwhen = 1979-05-27T07:32:00Z\nat = 2026-09-17T02:42:00Z\n```\nbody";
        let back: When = from_str(md).expect("toml date-times");
        assert_eq!(back.when, "1979-05-27T07:32:00Z");
        assert_eq!(
            back.at,
            buffa_types::google::protobuf::Timestamp::from_unix_secs(1_789_612_920)
        );
    }
}
