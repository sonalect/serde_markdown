//! Golden Markdown documents, hand-written structs, and constructors for
//! buffa-generated `markdown.testdata` messages.
//!
//! Protobuf sources live in `proto/markdown/testdata/`. Markdown goldens live
//! in `rust/serde_markdown/testdata/markdown/`. Hand-written structs are in
//! [`types`].

#![allow(dead_code)]

pub mod goldens;
pub mod types;
pub mod values;

use serde_markdown_generated::markdown::testdata as generated;

use crate::Markdown;

/// Manual [`Markdown`] for generated fixtures until annotate wiring (later).
impl Markdown for generated::Page {
    const BODY_FIELDS: &'static [&'static str] = &["text1", "appendix"];
}

/// Manual [`Markdown`] for generated fixtures until annotate wiring (later).
impl Markdown for generated::WellKnown {
    const BODY_FIELDS: &'static [&'static str] = &["occurredAt", "pause"];
}
