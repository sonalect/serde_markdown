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
