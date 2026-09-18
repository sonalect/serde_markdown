//! Golden Markdown documents, hand-written structs, and constructors for
//! buffa-generated `markdown.testdata` messages.
//!
//! Protobuf sources live in `proto/markdown/testdata/`. Markdown goldens live
//! in `rust/serde_markdown/testdata/markdown/`. Hand-written structs are in
//! [`types`].
//!
//! [`Markdown::BODY_FIELDS`] on generated fixtures uses proto3 JSON names,
//! matching what `#[derive(Markdown)]` plus `#[markdown(body)]` would emit
//! after buffa's `#[serde(rename = json_name)]`. This repository's buf
//! generate path does not inject those attributes (`buffa_build::Config` has
//! no `FileDescriptorSet` callback). `annotate_markdown_body` is for a
//! consumer `build.rs`. Views are not [`Markdown`].

pub mod goldens;
pub mod types;
pub mod values;

use serde_markdown_generated::markdown::testdata as generated;

use crate::Markdown;

impl Markdown for generated::Page {
    const BODY_FIELDS: &'static [&'static str] = &["text1", "appendix"];
}

impl Markdown for generated::WellKnown {
    const BODY_FIELDS: &'static [&'static str] = &["occurredAt", "pause"];
}

impl Markdown for generated::JsonNames {
    const BODY_FIELDS: &'static [&'static str] = &["bodyNote"];
}

impl Markdown for generated::OptionalBody {
    const BODY_FIELDS: &'static [&'static str] = &["first", "middle", "last"];
}

impl Markdown for generated::Article {
    const BODY_FIELDS: &'static [&'static str] = &["note"];
}

impl Markdown for generated::Proto3Page {
    const BODY_FIELDS: &'static [&'static str] = &["body"];
}

impl Markdown for generated::FieldsOnly {
    const BODY_FIELDS: &'static [&'static str] = &[];
}

impl Markdown for generated::BodyOnly {
    const BODY_FIELDS: &'static [&'static str] = &["text1", "text2"];
}

impl Markdown for generated::NestedFields {
    const BODY_FIELDS: &'static [&'static str] = &["body"];
}
