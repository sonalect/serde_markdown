//! Serialize a `Markdown` root struct to a document.

use serde::ser::{Impossible, Serialize, SerializeStruct, Serializer};
use serde_json::{Map, Value};

use crate::error::Error;
use crate::format::{self, FieldsLayout, Format};
use crate::markdown::Markdown;

/// Serialize `value` as a fenced YAML Markdown document.
///
/// The fields block is a labeled `yaml` fence. Body `String` fields and types
/// whose serde is a JSON string (well-known `Timestamp`, `Duration`,
/// `FieldMask`) are written as raw section text; nested objects and scalars
/// use the fence-format dump.
/// Sections are joined with `\n---\n`. Trailing `None` body fields are omitted;
/// a middle `None` is an empty section; `Some("")` is the two characters `""`.
/// There is no leading blank line, one newline after the closing fence, a
/// trailing newline at EOF, and no trailing `---` after the last emitted
/// section.
pub fn to_string<T: Serialize + Markdown>(value: &T) -> Result<String, Error> {
    to_string_with(value, Format::Yaml, FieldsLayout::Fenced)
}

/// Serialize `value` as a fenced Markdown document in `format`.
///
/// Same layout as [`to_string`]: a labeled fence (`yaml`, `json`, or `toml`)
/// then body sections. JSON is pretty-printed with 2-space indent; TOML uses
/// that crate's pretty printer.
pub fn to_string_with_format<T: Serialize + Markdown>(
    value: &T,
    format: Format,
) -> Result<String, Error> {
    to_string_with(value, format, FieldsLayout::Fenced)
}

/// Serialize `value` with an explicit fence language and fields layout.
///
/// [`FieldsLayout::Fenced`] wraps fields in a labeled code block.
/// [`FieldsLayout::Bare`] writes the mapping with no fence and a `---`
/// separator before the first body section. [`to_string`] is YAML and fenced.
pub fn to_string_with<T: Serialize + Markdown>(
    value: &T,
    format: Format,
    layout: FieldsLayout,
) -> Result<String, Error> {
    let captured = value.serialize(RootSerializer {
        body_fields: T::BODY_FIELDS,
    })?;
    render(captured, T::BODY_FIELDS, format, layout)
}

struct Captured {
    fields: Map<String, Value>,
    body: Map<String, Value>,
}

struct RootSerializer {
    body_fields: &'static [&'static str],
}

fn not_named_struct() -> Error {
    Error::type_error("root must be a named struct")
}

fn deny_root<T>() -> Result<T, Error> {
    Err(not_named_struct())
}

impl Serializer for RootSerializer {
    type Ok = Captured;
    type Error = Error;
    type SerializeSeq = Impossible<Captured, Error>;
    type SerializeTuple = Impossible<Captured, Error>;
    type SerializeTupleStruct = Impossible<Captured, Error>;
    type SerializeTupleVariant = Impossible<Captured, Error>;
    type SerializeMap = Impossible<Captured, Error>;
    type SerializeStruct = StructSerializer;
    type SerializeStructVariant = Impossible<Captured, Error>;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        deny_root()
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        deny_root()
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        deny_root()
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        deny_root()
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        deny_root()
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        deny_root()
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        deny_root()
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        deny_root()
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        deny_root()
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        deny_root()
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        deny_root()
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        deny_root()
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
        deny_root()
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        deny_root()
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(not_named_struct())
    }

    fn serialize_some<T: ?Sized + Serialize>(self, _value: &T) -> Result<Self::Ok, Self::Error> {
        Err(not_named_struct())
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(not_named_struct())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(not_named_struct())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(not_named_struct())
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        Err(not_named_struct())
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        Err(not_named_struct())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(not_named_struct())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(not_named_struct())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(not_named_struct())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(not_named_struct())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(not_named_struct())
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(StructSerializer {
            body_fields: self.body_fields,
            fields: Map::new(),
            body: Map::new(),
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(not_named_struct())
    }
}

struct StructSerializer {
    body_fields: &'static [&'static str],
    fields: Map<String, Value>,
    body: Map<String, Value>,
}

impl SerializeStruct for StructSerializer {
    type Ok = Captured;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        let json = serde_json::to_value(value).map_err(Error::type_error)?;
        if self.body_fields.contains(&key) {
            self.body.insert(key.to_owned(), json);
        } else {
            self.fields.insert(key.to_owned(), json);
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Captured {
            fields: self.fields,
            body: self.body,
        })
    }
}

fn render(
    captured: Captured,
    body_fields: &'static [&'static str],
    format: Format,
    layout: FieldsLayout,
) -> Result<String, Error> {
    let mut out = String::new();
    let has_fields = !captured.fields.is_empty();
    if has_fields {
        let dumped = format::dump(&Value::Object(captured.fields), format)?;
        match layout {
            FieldsLayout::Fenced => {
                out.push_str("```");
                out.push_str(fence_lang(format));
                out.push('\n');
                push_with_newline(&mut out, &dumped);
                out.push_str("```\n");
            }
            FieldsLayout::Bare => push_with_newline(&mut out, &dumped),
        }
    }

    let mut slots = Vec::with_capacity(body_fields.len());
    for name in body_fields {
        slots.push(body_slot(captured.body.get(*name), format)?);
    }
    while slots
        .pop_if(|slot| matches!(slot, BodySlot::Absent))
        .is_some()
    {}

    for (i, slot) in slots.iter().enumerate() {
        let need_sep = i > 0 || (has_fields && layout == FieldsLayout::Bare);
        if need_sep {
            if !out.ends_with('\n') {
                out.push('\n');
            }
            out.push_str("---\n");
        }
        match slot {
            BodySlot::Absent => {
                if need_sep {
                    out.push('\n');
                }
            }
            BodySlot::Raw(text) => out.push_str(text),
        }
    }

    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    Ok(out)
}

enum BodySlot {
    Absent,
    Raw(String),
}

fn body_slot(value: Option<&Value>, format: Format) -> Result<BodySlot, Error> {
    match value {
        None | Some(Value::Null) => Ok(BodySlot::Absent),
        Some(Value::String(s)) if s.is_empty() => Ok(BodySlot::Raw("\"\"".to_owned())),
        Some(Value::String(s)) => Ok(BodySlot::Raw(s.clone())),
        Some(other) => Ok(BodySlot::Raw(format::dump(other, format)?)),
    }
}

fn fence_lang(format: Format) -> &'static str {
    match format {
        Format::Yaml => "yaml",
        Format::Json => "json",
        Format::Toml => "toml",
    }
}

fn push_with_newline(out: &mut String, text: &str) {
    out.push_str(text);
    if !out.ends_with('\n') {
        out.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use super::{to_string, to_string_with, to_string_with_format};
    use crate::error::ErrorKind;
    use crate::format::{FieldsLayout, Format};
    use crate::markdown::Markdown;
    use crate::testdata::{goldens, types, values};

    #[derive(Serialize)]
    struct TupleRoot(i32);

    impl Markdown for TupleRoot {
        const BODY_FIELDS: &'static [&'static str] = &[];
    }

    #[derive(Serialize)]
    struct UnitRoot;

    impl Markdown for UnitRoot {
        const BODY_FIELDS: &'static [&'static str] = &[];
    }

    #[test]
    fn page_round_trip() {
        let page = values::page();
        let md = to_string(&page).expect("serialize");
        let back: types::Page = crate::from_str(&md).expect("deserialize");
        assert_eq!(page, back);
    }

    #[test]
    fn serialize_is_labeled_yaml_fenced() {
        let md = to_string(&values::page()).expect("serialize");
        assert!(md.starts_with("```yaml\n"), "{md}");
        assert!(!md.starts_with('\n'), "{md}");
        assert!(md.ends_with('\n'), "{md}");
        assert!(md.contains("```\nText1 bla bla bla\n---\n"), "{md}");
        assert!(!md.contains("```\n\n"), "{md}");
        assert!(!md.contains("```yml"), "{md}");
        assert!(md.contains("field1: foo"), "{md}");
        assert!(md.contains("field2: bar"), "{md}");
        assert!(md.contains("field3: 1"), "{md}");
        assert!(md.contains("Text1 bla bla bla"), "{md}");
        assert!(md.contains("Text2 bal bla bla"), "{md}");
        assert!(
            !md.contains("\"Text1 bla bla bla\""),
            "body strings must be raw, got {md}"
        );
        let trimmed = md.trim_end();
        assert!(!trimmed.ends_with("---"), "{md}");
    }

    #[test]
    fn fields_only_omits_body() {
        let md = to_string(&values::fields_only()).expect("serialize");
        assert!(md.starts_with("```yaml\n"), "{md}");
        assert!(md.contains("name: only"), "{md}");
        assert!(md.contains("count: 2"), "{md}");
        assert!(!md.contains("---"), "{md}");
        assert!(!md.contains("Text1"), "{md}");
        let back: types::FieldsOnly = crate::from_str(&md).expect("deserialize");
        assert_eq!(values::fields_only(), back);
    }

    #[test]
    fn deserialize_fenced_yaml_golden() {
        let page: types::Page =
            crate::from_str(goldens::PAGE_FENCED_YAML).expect("page.fenced.yaml.md");
        assert_eq!(page, values::page());
    }

    #[cfg(feature = "json")]
    #[test]
    fn serialize_json_is_labeled_pretty_fence() {
        let md = to_string_with_format(&values::page(), Format::Json).expect("serialize");
        assert!(md.starts_with("```json\n"), "{md}");
        assert!(md.contains("{\n  \"field1\": \"foo\""), "{md}");
        assert!(md.contains("  \"field2\": \"bar\""), "{md}");
        assert!(md.contains("  \"field3\": 1"), "{md}");
        assert!(md.contains("```\nText1 bla bla bla\n---\n"), "{md}");
        assert!(!md.contains("```\n\n"), "{md}");
        let back: types::Page = crate::from_str(&md).expect("deserialize");
        assert_eq!(values::page(), back);
    }

    #[cfg(feature = "toml")]
    #[test]
    fn serialize_toml_is_labeled_pretty_fence() {
        let md = to_string_with_format(&values::page(), Format::Toml).expect("serialize");
        assert!(md.starts_with("```toml\n"), "{md}");
        assert!(md.contains("field1 = \"foo\""), "{md}");
        assert!(md.contains("field2 = \"bar\""), "{md}");
        assert!(md.contains("field3 = 1"), "{md}");
        assert!(md.contains("```\nText1 bla bla bla\n---\n"), "{md}");
        let back: types::Page = crate::from_str(&md).expect("deserialize");
        assert_eq!(values::page(), back);
    }

    #[test]
    fn serialize_bare_yaml_has_no_fence() {
        let md =
            to_string_with(&values::page(), Format::Yaml, FieldsLayout::Bare).expect("serialize");
        assert!(!md.contains("```"), "{md}");
        assert!(!md.starts_with('\n'), "{md}");
        assert!(md.ends_with('\n'), "{md}");
        assert!(md.contains("field1: foo"), "{md}");
        assert!(md.contains("field2: bar"), "{md}");
        assert!(md.contains("field3: 1"), "{md}");
        assert!(md.contains("---\nText1 bla bla bla\n---\n"), "{md}");
        let trimmed = md.trim_end();
        assert!(!trimmed.ends_with("---"), "{md}");
        let back: types::Page = crate::from_str(&md).expect("deserialize");
        assert_eq!(values::page(), back);
    }

    #[cfg(feature = "json")]
    #[test]
    fn serialize_bare_json_separator_before_body() {
        let md =
            to_string_with(&values::page(), Format::Json, FieldsLayout::Bare).expect("serialize");
        assert!(!md.contains("```"), "{md}");
        assert!(md.contains("{\n  \"field1\": \"foo\""), "{md}");
        assert!(md.contains("}\n---\nText1 bla bla bla\n---\n"), "{md}");
        let back: types::Page = crate::from_str(&md).expect("deserialize");
        assert_eq!(values::page(), back);
    }

    #[cfg(feature = "toml")]
    #[test]
    fn serialize_bare_toml_separator_before_body() {
        let md =
            to_string_with(&values::page(), Format::Toml, FieldsLayout::Bare).expect("serialize");
        assert!(!md.contains("```"), "{md}");
        assert!(md.contains("field1 = \"foo\""), "{md}");
        assert!(md.contains("field3 = 1"), "{md}");
        assert!(md.contains("---\nText1 bla bla bla\n---\n"), "{md}");
        let back: types::Page = crate::from_str(&md).expect("deserialize");
        assert_eq!(values::page(), back);
    }

    #[test]
    fn serialize_bare_fields_only_omits_separator() {
        let md = to_string_with(&values::fields_only(), Format::Yaml, FieldsLayout::Bare)
            .expect("serialize");
        assert!(!md.contains("```"), "{md}");
        assert!(md.contains("name: only"), "{md}");
        assert!(md.contains("count: 2"), "{md}");
        assert!(!md.contains("---"), "{md}");
        let back: types::FieldsOnly = crate::from_str(&md).expect("deserialize");
        assert_eq!(values::fields_only(), back);
    }

    fn assert_optional_round_trip(value: &types::OptionalBody, golden: &str) {
        let from_golden: types::OptionalBody = crate::from_str(golden).expect("golden");
        assert_eq!(*value, from_golden);
        let md = to_string(value).expect("serialize");
        let back: types::OptionalBody = crate::from_str(&md).expect("round-trip");
        assert_eq!(*value, back);
    }

    #[test]
    fn optional_middle_none_keeps_empty_slot() {
        let value = values::optional_middle_none();
        assert_optional_round_trip(&value, goldens::OPTIONAL_MIDDLE_NONE);
        let md = to_string(&value).expect("serialize");
        assert!(md.contains("```\na\n---\n\n---\nc\n"), "{md}");
    }

    #[test]
    fn optional_trailing_none_omits_separator() {
        let value = values::optional_trailing_none();
        assert_optional_round_trip(&value, goldens::OPTIONAL_TRAILING_NONE);
        let md = to_string(&value).expect("serialize");
        assert!(md.contains("```\na\n"), "{md}");
        assert!(!md.contains("---"), "{md}");
        let trimmed = md.trim_end();
        assert!(!trimmed.ends_with("---"), "{md}");
    }

    #[test]
    fn optional_leading_none_empty_first_section() {
        let value = values::optional_leading_none();
        assert_optional_round_trip(&value, goldens::OPTIONAL_LEADING_NONE);
        let md = to_string(&value).expect("serialize");
        assert!(md.contains("```\n---\na\n"), "{md}");
    }

    #[test]
    fn optional_empty_string_writes_quotes() {
        let value = values::optional_empty_string();
        assert_optional_round_trip(&value, goldens::OPTIONAL_EMPTY_STRING);
        let md = to_string(&value).expect("serialize");
        assert!(md.contains("```\na\n---\n\"\"\n"), "{md}");
    }

    #[test]
    fn to_string_stays_fenced_yaml() {
        let md = to_string(&values::page()).expect("serialize");
        assert!(md.starts_with("```yaml\n"), "{md}");
        let with_format = to_string_with_format(&values::page(), Format::Yaml).expect("yaml");
        assert_eq!(md, with_format);
    }

    #[cfg(not(feature = "json"))]
    #[test]
    fn json_without_feature_is_format_disabled() {
        let err = to_string_with_format(&values::page(), Format::Json).expect_err("disabled");
        assert_eq!(err.kind(), ErrorKind::FormatDisabled);
        assert!(std::error::Error::source(&err).is_none());
    }

    #[cfg(not(feature = "toml"))]
    #[test]
    fn toml_without_feature_is_format_disabled() {
        let err = to_string_with_format(&values::page(), Format::Toml).expect_err("disabled");
        assert_eq!(err.kind(), ErrorKind::FormatDisabled);
        assert!(std::error::Error::source(&err).is_none());
    }

    #[test]
    fn root_tuple_struct_is_type() {
        let err = to_string(&TupleRoot(1)).expect_err("tuple root");
        assert_eq!(err.kind(), ErrorKind::Type);
    }

    #[test]
    fn root_unit_struct_is_type() {
        let err = to_string(&UnitRoot).expect_err("unit root");
        assert_eq!(err.kind(), ErrorKind::Type);
        assert_eq!(err.to_string(), "root must be a named struct");
    }

    #[test]
    fn nested_round_trip() {
        let value = values::nested();
        let from_golden: types::NestedFields =
            crate::from_str(goldens::NESTED_FENCED_YAML).expect("golden");
        assert_eq!(value, from_golden);
        let md = to_string(&value).expect("serialize");
        assert!(md.contains("author: Ada"), "{md}");
        assert!(md.contains("draft: true"), "{md}");
        let back: types::NestedFields = crate::from_str(&md).expect("round-trip");
        assert_eq!(value, back);
    }

    #[test]
    fn structured_body_is_fence_format_dump() {
        let value = values::article();
        let from_golden: types::Article =
            crate::from_str(goldens::STRUCTURED_FENCED_YAML).expect("golden");
        assert_eq!(value, from_golden);
        let md = to_string(&value).expect("serialize");
        assert!(md.contains("```yaml\n"), "{md}");
        assert!(md.contains("title: Hello"), "{md}");
        assert!(md.contains("heading: Intro"), "{md}");
        assert!(md.contains("pages: 3"), "{md}");
        let back: types::Article = crate::from_str(&md).expect("round-trip");
        assert_eq!(value, back);
    }

    #[cfg(feature = "json")]
    #[test]
    fn structured_json_round_trip() {
        let value = values::article();
        let md = to_string_with_format(&value, Format::Json).expect("serialize");
        assert!(md.contains("```json\n"), "{md}");
        assert!(md.contains("\"heading\": \"Intro\""), "{md}");
        let back: types::Article = crate::from_str(&md).expect("round-trip");
        assert_eq!(value, back);
    }

    #[test]
    fn serde_rename_keys_in_markdown() {
        let value = values::json_names();
        let from_golden: types::JsonNames =
            crate::from_str(goldens::NAMES_FENCED_YAML).expect("golden");
        assert_eq!(value, from_golden);
        let md = to_string(&value).expect("serialize");
        assert!(md.contains("publishedAt:"), "{md}");
        assert!(!md.contains("published_at"), "{md}");
        assert!(md.contains("A body note."), "{md}");
        assert!(!md.contains("body_note"), "{md}");
        assert!(
            !md.contains("bodyNote:"),
            "body field name must not appear as a key, got {md}"
        );
        let back: types::JsonNames = crate::from_str(&md).expect("round-trip");
        assert_eq!(value, back);
    }

    #[test]
    fn whitespace_round_trip_keeps_interiors() {
        let value = values::whitespace_page();
        let md = to_string(&value).expect("serialize");
        assert!(md.contains("spaces   \n"), "{md}");
        assert!(md.contains("🦀"), "{md}");
        let back: types::Page = crate::from_str(&md).expect("round-trip");
        assert_eq!(value, back);
    }

    #[test]
    fn well_known_round_trip() {
        let value = values::well_known();
        let from_golden: types::WellKnown =
            crate::from_str(goldens::WELL_KNOWN_FENCED_YAML).expect("golden");
        assert_eq!(value, from_golden);
        let md = to_string(&value).expect("serialize");
        assert!(md.contains("published:"), "{md}");
        assert!(md.contains(values::PUBLISHED_RFC3339), "{md}");
        assert!(
            md.contains("1.5s") || md.contains("1.500s"),
            "Duration proto3 JSON (1.5s or 1.500s), got {md}"
        );
        assert!(md.contains("meta:"), "{md}");
        assert!(md.contains("{}"), "empty Empty object, got {md}");
        assert!(md.contains("lang:"), "{md}");
        assert!(md.contains("a,b.c"), "{md}");
        assert!(md.contains("count:"), "{md}");
        assert!(md.contains("3"), "{md}");
        assert!(
            md.contains("```\n2026-09-17T02:42:00Z\n---\n"),
            "Timestamp body must be raw RFC 3339 after the fence, got {md}"
        );
        assert!(
            md.contains("---\n1.5s\n") || md.contains("---\n1.500s\n"),
            "Duration body must be raw proto3 JSON, got {md}"
        );
        assert!(
            !md.contains("\"2026-09-17T02:42:00Z\"\n---"),
            "body Timestamp must not be a quoted JSON string, got {md}"
        );
        let back: types::WellKnown = crate::from_str(&md).expect("round-trip");
        assert_eq!(value, back);
    }

    #[test]
    fn well_known_generated_round_trip() {
        let value = values::well_known_generated();
        let from_golden: serde_markdown_generated::markdown::testdata::WellKnown =
            crate::from_str(goldens::WELL_KNOWN_FENCED_YAML).expect("golden");
        assert_eq!(value, from_golden);
        let md = to_string(&value).expect("serialize");
        let back: serde_markdown_generated::markdown::testdata::WellKnown =
            crate::from_str(&md).expect("round-trip");
        assert_eq!(value, back);
    }

    #[test]
    fn page_published_round_trip() {
        let value = values::page_with_published();
        let from_golden: types::Page =
            crate::from_str(goldens::PAGE_PUBLISHED_YAML).expect("golden");
        assert_eq!(value, from_golden);
        let md = to_string(&value).expect("serialize");
        assert!(md.contains("published:"), "{md}");
        assert!(md.contains(values::PUBLISHED_RFC3339), "{md}");
        let back: types::Page = crate::from_str(&md).expect("round-trip");
        assert_eq!(value, back);
    }

    #[test]
    fn page_published_generated_round_trip() {
        let value = values::page_generated_with_published();
        let from_golden: serde_markdown_generated::markdown::testdata::Page =
            crate::from_str(goldens::PAGE_PUBLISHED_YAML).expect("golden");
        assert_eq!(value, from_golden);
        let md = to_string(&value).expect("serialize");
        let back: serde_markdown_generated::markdown::testdata::Page =
            crate::from_str(&md).expect("round-trip");
        assert_eq!(value, back);
    }
}
