//! Hand-written structs that mirror `proto/markdown/testdata`.
//!
//! `BODY_FIELDS` lists Serde field names (after `rename`) in declaration
//! order, the same contract `serde_markdown::Markdown` will use.

use std::collections::BTreeMap;

use buffa_types::google::protobuf::{Duration, Timestamp};
use serde::{Deserialize, Serialize};

/// DESIGN.md §2 / §3.1 example (`markdown.testdata.Page`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Page {
    pub field1: String,
    pub field2: String,
    pub field3: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published: Option<Timestamp>,
    pub text1: String,
    pub appendix: String,
}

impl Page {
    pub const BODY_FIELDS: &'static [&'static str] = &["text1", "appendix"];
}

/// No body sections (`markdown.testdata.FieldsOnly`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldsOnly {
    pub name: String,
    pub count: i32,
}

impl FieldsOnly {
    pub const BODY_FIELDS: &'static [&'static str] = &[];
}

/// Body-only document (`markdown.testdata.BodyOnly`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BodyOnly {
    pub text1: String,
    pub text2: String,
}

impl BodyOnly {
    pub const BODY_FIELDS: &'static [&'static str] = &["text1", "text2"];
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Meta {
    pub author: String,
    pub tags: Vec<String>,
}

/// Nested front matter (`markdown.testdata.NestedFields`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NestedFields {
    pub meta: Meta,
    pub draft: bool,
    pub body: String,
}

impl NestedFields {
    pub const BODY_FIELDS: &'static [&'static str] = &["body"];
}

/// Presence matrix (`markdown.testdata.OptionalBody`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionalBody {
    pub title: String,
    pub first: Option<String>,
    pub middle: Option<String>,
    pub last: Option<String>,
}

impl OptionalBody {
    pub const BODY_FIELDS: &'static [&'static str] = &["first", "middle", "last"];
}

/// Snake_case identifiers, camelCase Markdown keys (`markdown.testdata.JsonNames`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonNames {
    #[serde(rename = "publishedAt")]
    pub published_at: String,
    #[serde(rename = "bodyNote")]
    pub body_note: String,
}

impl JsonNames {
    pub const BODY_FIELDS: &'static [&'static str] = &["bodyNote"];
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    pub heading: String,
    pub pages: i32,
}

/// Structured body section (`markdown.testdata.Article`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Article {
    pub title: String,
    pub note: Note,
}

impl Article {
    pub const BODY_FIELDS: &'static [&'static str] = &["note"];
}

/// WKT in fields and body (`markdown.testdata.WellKnown`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WellKnown {
    pub published: Timestamp,
    pub ttl: Duration,
    pub extra: BTreeMap<String, String>,
    pub mask: String,
    pub count: i32,
    #[serde(rename = "occurredAt")]
    pub occurred_at: Timestamp,
    pub pause: Duration,
}

impl WellKnown {
    pub const BODY_FIELDS: &'static [&'static str] = &["occurredAt", "pause"];
}

/// proto3 `optional` body (`markdown.testdata.Proto3Page`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proto3Page {
    pub title: String,
    pub body: Option<String>,
}

impl Proto3Page {
    pub const BODY_FIELDS: &'static [&'static str] = &["body"];
}
