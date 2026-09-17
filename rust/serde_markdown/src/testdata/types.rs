//! Hand-written structs that mirror `proto/markdown/testdata`.
//!
//! [`Markdown::BODY_FIELDS`] lists Serde field names (after `rename`) in
//! declaration order.

use std::collections::BTreeMap;

use buffa_types::google::protobuf::{Duration, Timestamp};
use serde::{Deserialize, Serialize};

use crate::Markdown;

/// Fixture page (`markdown.testdata.Page`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Markdown)]
pub struct Page {
    pub field1: String,
    pub field2: String,
    pub field3: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published: Option<Timestamp>,
    #[markdown(body)]
    pub text1: String,
    #[markdown(body)]
    pub appendix: String,
}

/// No body sections (`markdown.testdata.FieldsOnly`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Markdown)]
pub struct FieldsOnly {
    pub name: String,
    pub count: i32,
}

/// Body-only document (`markdown.testdata.BodyOnly`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Markdown)]
pub struct BodyOnly {
    #[markdown(body)]
    pub text1: String,
    #[markdown(body)]
    pub text2: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Meta {
    pub author: String,
    pub tags: Vec<String>,
}

/// Nested front matter (`markdown.testdata.NestedFields`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Markdown)]
pub struct NestedFields {
    pub meta: Meta,
    pub draft: bool,
    #[markdown(body)]
    pub body: String,
}

/// Presence matrix (`markdown.testdata.OptionalBody`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Markdown)]
pub struct OptionalBody {
    pub title: String,
    #[markdown(body)]
    pub first: Option<String>,
    #[markdown(body)]
    pub middle: Option<String>,
    #[markdown(body)]
    pub last: Option<String>,
}

/// Snake_case identifiers, camelCase Markdown keys (`markdown.testdata.JsonNames`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Markdown)]
pub struct JsonNames {
    #[serde(rename = "publishedAt")]
    pub published_at: String,
    #[serde(rename = "bodyNote")]
    #[markdown(body)]
    pub body_note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    pub heading: String,
    pub pages: i32,
}

/// Structured body section (`markdown.testdata.Article`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Markdown)]
pub struct Article {
    pub title: String,
    #[markdown(body)]
    pub note: Note,
}

/// WKT in fields and body (`markdown.testdata.WellKnown`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Markdown)]
pub struct WellKnown {
    pub published: Timestamp,
    pub ttl: Duration,
    pub extra: BTreeMap<String, String>,
    pub mask: String,
    pub count: i32,
    #[serde(rename = "occurredAt")]
    #[markdown(body)]
    pub occurred_at: Timestamp,
    #[markdown(body)]
    pub pause: Duration,
}

/// proto3 `optional` body (`markdown.testdata.Proto3Page`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Markdown)]
pub struct Proto3Page {
    pub title: String,
    #[markdown(body)]
    pub body: Option<String>,
}
