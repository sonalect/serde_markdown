//! Expected values that pair with the Markdown goldens.

use buffa_types::google::protobuf::{
    Duration, Empty, FieldMask, Int32Value, Struct, Timestamp, Value,
};
use serde_markdown_generated::markdown::testdata::{
    Article, BodyOnly, FieldsOnly, JsonNames, NestedFields, Note, OptionalBody, Page, Proto3Page,
    WellKnown, nested_fields,
};

use super::types;

/// RFC 3339 value used in `page.published.yaml.md` and WKT goldens.
pub const PUBLISHED_RFC3339: &str = "2026-09-17T02:42:00Z";
/// Unix seconds for [`PUBLISHED_RFC3339`].
pub const PUBLISHED_UNIX: i64 = 1_789_612_920;
pub const TEXT1: &str = "Text1 bla bla bla";
pub const TEXT2: &str = "Text2 bal bla bla";

fn published() -> Timestamp {
    Timestamp::from_unix_secs(PUBLISHED_UNIX)
}

fn ttl_1_5s() -> Duration {
    Duration {
        seconds: 1,
        nanos: 500_000_000,
        ..Default::default()
    }
}

pub fn page() -> types::Page {
    types::Page {
        field1: "foo".into(),
        field2: "bar".into(),
        field3: 1,
        published: None,
        text1: TEXT1.into(),
        appendix: TEXT2.into(),
    }
}

pub fn page_with_published() -> types::Page {
    types::Page {
        published: Some(published()),
        ..page()
    }
}

pub fn page_generated() -> Page {
    Page::default()
        .with_field1("foo")
        .with_field2("bar")
        .with_field3(1)
        .with_text1(TEXT1)
        .with_appendix(TEXT2)
}

pub fn page_generated_with_published() -> Page {
    let mut page = page_generated();
    page.published = published().into();
    page
}

pub fn fields_only() -> types::FieldsOnly {
    types::FieldsOnly {
        name: "only".into(),
        count: 2,
    }
}

pub fn fields_only_generated() -> FieldsOnly {
    FieldsOnly::default().with_name("only").with_count(2)
}

pub fn body_only() -> types::BodyOnly {
    types::BodyOnly {
        text1: TEXT1.into(),
        text2: TEXT2.into(),
    }
}

pub fn body_only_generated() -> BodyOnly {
    BodyOnly::default().with_text1(TEXT1).with_text2(TEXT2)
}

pub fn nested() -> types::NestedFields {
    types::NestedFields {
        meta: types::Meta {
            author: "Ada".into(),
            tags: vec!["rust".into(), "markdown".into()],
        },
        draft: true,
        body: "The article body.".into(),
    }
}

pub fn nested_generated() -> NestedFields {
    let mut meta = nested_fields::Meta::default().with_author("Ada");
    meta.tags = vec!["rust".into(), "markdown".into()];
    let mut msg = NestedFields::default()
        .with_draft(true)
        .with_body("The article body.");
    msg.meta = meta.into();
    msg
}

pub fn optional_middle_none() -> types::OptionalBody {
    types::OptionalBody {
        title: "memo".into(),
        first: Some("a".into()),
        middle: None,
        last: Some("c".into()),
    }
}

pub fn optional_trailing_none() -> types::OptionalBody {
    types::OptionalBody {
        title: "memo".into(),
        first: Some("a".into()),
        middle: None,
        last: None,
    }
}

pub fn optional_leading_none() -> types::OptionalBody {
    types::OptionalBody {
        title: "memo".into(),
        first: None,
        middle: Some("a".into()),
        last: None,
    }
}

pub fn optional_empty_string() -> types::OptionalBody {
    types::OptionalBody {
        title: "memo".into(),
        first: Some("a".into()),
        middle: Some(String::new()),
        last: None,
    }
}

pub fn optional_middle_none_generated() -> OptionalBody {
    OptionalBody::default()
        .with_title("memo")
        .with_first("a")
        .with_last("c")
}

pub fn optional_trailing_none_generated() -> OptionalBody {
    OptionalBody::default().with_title("memo").with_first("a")
}

pub fn optional_leading_none_generated() -> OptionalBody {
    OptionalBody::default().with_title("memo").with_middle("a")
}

pub fn optional_empty_string_generated() -> OptionalBody {
    OptionalBody::default()
        .with_title("memo")
        .with_first("a")
        .with_middle("")
}

pub fn whitespace_page() -> types::Page {
    types::Page {
        text1: concat!(
            "Line with trailing spaces   \n",
            "and a blank line in between.\n",
            "\n",
            "Ещё кириллица и emoji 🦀."
        )
        .into(),
        ..page()
    }
}

pub fn json_names() -> types::JsonNames {
    types::JsonNames {
        published_at: PUBLISHED_RFC3339.into(),
        body_note: "A body note.".into(),
    }
}

pub fn json_names_generated() -> JsonNames {
    JsonNames::default()
        .with_published_at(PUBLISHED_RFC3339)
        .with_body_note("A body note.")
}

pub fn article() -> types::Article {
    types::Article {
        title: "Hello".into(),
        note: types::Note {
            heading: "Intro".into(),
            pages: 3,
        },
    }
}

pub fn article_generated() -> Article {
    let mut article = Article::default().with_title("Hello");
    article.note = Note::default().with_heading("Intro").with_pages(3).into();
    article
}

pub fn well_known() -> types::WellKnown {
    types::WellKnown {
        published: published(),
        ttl: ttl_1_5s(),
        extra: [("lang".into(), "ru".into())].into(),
        mask: "a,b.c".into(),
        count: 3,
        occurred_at: published(),
        pause: ttl_1_5s(),
    }
}

pub fn well_known_generated() -> WellKnown {
    let mut extra = Struct::default();
    extra.fields.insert("lang".into(), Value::from("ru"));
    WellKnown {
        published: published().into(),
        ttl: ttl_1_5s().into(),
        meta: Empty::default().into(),
        extra: extra.into(),
        mask: FieldMask {
            paths: vec!["a".into(), "b.c".into()],
            ..Default::default()
        }
        .into(),
        count: Int32Value {
            value: 3,
            ..Default::default()
        }
        .into(),
        occurred_at: published().into(),
        pause: ttl_1_5s().into(),
        ..Default::default()
    }
}

pub fn proto3_page() -> types::Proto3Page {
    types::Proto3Page {
        title: "hello".into(),
        body: Some("body text".into()),
    }
}

pub fn proto3_page_generated() -> Proto3Page {
    let mut page = Proto3Page::default().with_body("body text");
    page.title = "hello".into();
    page
}
