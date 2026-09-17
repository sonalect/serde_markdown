//! Deserialize a `Markdown` root struct from a document.

use std::vec;

use serde::de::{
    DeserializeOwned, DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor,
};
use serde::de::value::{StringDeserializer, StrDeserializer};
use serde_json::{Map, Value};

use crate::error::{Error, ErrorKind};
use crate::format::{self, Format};
use crate::markdown::Markdown;
use crate::parse::{self, Document, Fields, Sniff};

/// Deserialize a Markdown document from `s`.
///
/// The optional fields block is the first fenced `yaml` / `yml` / `json` /
/// `toml` code block, or a **bare** mapping before the first top-level `---`.
/// An unlabeled first fence and a bare first slice are sniffed: `{` / `[`
/// means JSON, a TOML-shaped first line means TOML, otherwise YAML. A sniffed
/// JSON or TOML parse failure is [`ErrorKind::FrontMatter`] and does not fall
/// back to YAML. A bare first slice that is not a mapping is the first body
/// section. Leading Unicode whitespace and top-level `---` before fields are
/// discarded. A fenced `yaml` block that is not first is body.
///
/// Body fields listed in [`Markdown::BODY_FIELDS`] are filled from sections
/// in declaration order. `String` / bytes sections are the raw section text
/// (interiors are not trimmed). Nested structs, maps, sequences, and scalars
/// parse as the fields format when a fields block was present, otherwise YAML.
///
/// Trailing omitted sections become `None` on optional fields. An empty
/// section is `None`; a section whose content is the two characters `""`
/// is `Some("")`. Extra sections, a missing required section, and an empty
/// section for a required structured field are [`ErrorKind::Body`]. A decoder
/// failure on a structured section is [`ErrorKind::Body`] with `source` set.
pub fn from_str<T>(s: &str) -> Result<T, Error>
where
    T: DeserializeOwned + Markdown,
{
    let decoded = decode_document(parse::parse(s))?;
    if decoded.body.len() > T::BODY_FIELDS.len() {
        return Err(Error::body(format!(
            "expected at most {} body section(s), found {}",
            T::BODY_FIELDS.len(),
            decoded.body.len()
        )));
    }
    let slots = section_slots(T::BODY_FIELDS.len(), &decoded.body);
    serde::de::Deserialize::deserialize(RootDe {
        fields: decoded.fields,
        body_fields: T::BODY_FIELDS,
        slots,
        format: decoded.format,
    })
}

struct Decoded {
    fields: Map<String, Value>,
    body: Vec<String>,
    format: Format,
}

fn decode_document(doc: Document) -> Result<Decoded, Error> {
    let Document { fields, body } = doc;
    match fields {
        Fields::Fenced {
            labeled,
            sniff,
            inner,
        } => {
            let format = sniff_format(labeled.unwrap_or(sniff));
            let map = mapping_or_type_error(format::load(&inner, format)?)?;
            Ok(Decoded {
                fields: map,
                body,
                format,
            })
        }
        Fields::Bare { sniff, inner } => match load_bare_mapping(&inner, sniff)? {
            Some(map) => Ok(Decoded {
                fields: map,
                body,
                format: sniff_format(sniff),
            }),
            None => {
                let mut sections = Vec::with_capacity(body.len() + 1);
                sections.push(inner);
                sections.extend(body);
                Ok(Decoded {
                    fields: Map::new(),
                    body: sections,
                    format: Format::Yaml,
                })
            }
        },
    }
}

/// Decode a **fenced** fields block. Not a mapping is a type error.
fn mapping_or_type_error(value: Value) -> Result<Map<String, Value>, Error> {
    match mapping_from_value(value) {
        Some(map) => Ok(map),
        None => Err(Error::type_error("fields block must be a mapping")),
    }
}

fn mapping_from_value(value: Value) -> Option<Map<String, Value>> {
    match value {
        Value::Object(map) => Some(map),
        Value::Null => Some(Map::new()),
        _ => None,
    }
}

/// Bare first slice: mapping → fields. YAML that is not a mapping (or that
/// the YAML decoder rejects) → `None` (treat as body). JSON/TOML decode
/// failure stays [`ErrorKind::FrontMatter`].
fn load_bare_mapping(inner: &str, sniff: Sniff) -> Result<Option<Map<String, Value>>, Error> {
    let format = sniff_format(sniff);
    let value = match format::load(inner, format) {
        Ok(value) => value,
        Err(err) if err.kind() == ErrorKind::FormatDisabled => return Err(err),
        Err(err) if matches!(format, Format::Json | Format::Toml) => return Err(err),
        Err(_) => return Ok(None),
    };
    Ok(mapping_from_value(value))
}

fn sniff_format(sniff: Sniff) -> Format {
    match sniff {
        Sniff::Yaml => Format::Yaml,
        Sniff::Json => Format::Json,
        Sniff::Toml => Format::Toml,
    }
}

/// Two characters that encode optional `Some("")` (distinct from an empty
/// section, which is `None`).
const EMPTY_STRING_SECTION: &str = "\"\"";

#[derive(Clone)]
enum SectionSlot {
    Absent,
    Empty,
    Text(String),
}

fn section_slots(n: usize, sections: &[String]) -> Vec<SectionSlot> {
    (0..n)
        .map(|i| match sections.get(i) {
            None => SectionSlot::Absent,
            Some(section) => {
                let text = section_text(section);
                if text.is_empty() {
                    SectionSlot::Empty
                } else {
                    SectionSlot::Text(text.to_owned())
                }
            }
        })
        .collect()
}

/// Drop the document line terminator after a section. Interior blank lines and
/// trailing spaces stay in the section text.
fn section_text(section: &str) -> &str {
    match section.strip_suffix("\r\n") {
        Some(s) => s,
        None => match section.strip_suffix('\n') {
            Some(s) => s,
            None => section,
        },
    }
}

fn into_body(err: Error) -> Error {
    match err.kind() {
        ErrorKind::Body | ErrorKind::FormatDisabled | ErrorKind::Syntax => err,
        _ => Error::body(err),
    }
}

struct RootDe {
    fields: Map<String, Value>,
    body_fields: &'static [&'static str],
    slots: Vec<SectionSlot>,
    format: Format,
}

impl<'de> Deserializer<'de> for RootDe {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_map(visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_map(RootMap::new(self))
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.deserialize_map(visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_unit()
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct enum identifier
    }
}

enum Pending {
    Front(String),
    Body { name: &'static str, index: usize },
}

struct RootMap {
    fields: Map<String, Value>,
    slots: Vec<SectionSlot>,
    format: Format,
    pending: vec::IntoIter<Pending>,
    current: Option<Pending>,
}

impl RootMap {
    fn new(root: RootDe) -> Self {
        let mut pending = Vec::with_capacity(root.fields.len() + root.body_fields.len());
        for key in root.fields.keys() {
            if !root.body_fields.contains(&key.as_str()) {
                pending.push(Pending::Front(key.clone()));
            }
        }
        for (index, name) in root.body_fields.iter().enumerate() {
            pending.push(Pending::Body { name, index });
        }
        Self {
            fields: root.fields,
            slots: root.slots,
            format: root.format,
            pending: pending.into_iter(),
            current: None,
        }
    }
}

impl<'de> MapAccess<'de> for RootMap {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: DeserializeSeed<'de>,
    {
        match self.pending.next() {
            None => Ok(None),
            Some(Pending::Front(key)) => {
                self.current = Some(Pending::Front(key.clone()));
                seed.deserialize(StringDeserializer::<Error>::new(key))
                    .map(Some)
            }
            Some(Pending::Body { name, index }) => {
                self.current = Some(Pending::Body { name, index });
                seed.deserialize(StrDeserializer::<Error>::new(name))
                    .map(Some)
            }
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
    where
        V: DeserializeSeed<'de>,
    {
        match self.current.take() {
            Some(Pending::Front(key)) => {
                let value = self.fields.remove(&key).unwrap_or(Value::Null);
                seed.deserialize(JsonDe(value))
            }
            Some(Pending::Body { index, .. }) => {
                let slot = self
                    .slots
                    .get(index)
                    .cloned()
                    .unwrap_or(SectionSlot::Absent);
                seed.deserialize(SectionDeserializer {
                    slot,
                    format: self.format,
                })
            }
            None => Err(Error::type_error("value is missing")),
        }
    }
}

struct SectionDeserializer {
    slot: SectionSlot,
    format: Format,
}

impl SectionDeserializer {
    fn parsed(self) -> Result<JsonDe, Error> {
        match self.slot {
            SectionSlot::Absent => Err(Error::body("missing section")),
            SectionSlot::Empty => Err(Error::body("empty section for required structured field")),
            SectionSlot::Text(text) => Ok(JsonDe(format::load_body(&text, self.format)?)),
        }
    }
}

impl<'de> Deserializer<'de> for SectionDeserializer {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.parsed()?.deserialize_any(visitor).map_err(into_body)
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.slot {
            SectionSlot::Absent => Err(Error::body("missing section")),
            SectionSlot::Empty => visitor.visit_string(String::new()),
            SectionSlot::Text(text) if text == EMPTY_STRING_SECTION => {
                visitor.visit_string(String::new())
            }
            SectionSlot::Text(text) => visitor.visit_string(text),
        }
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.slot {
            SectionSlot::Absent => Err(Error::body("missing section")),
            SectionSlot::Empty => visitor.visit_byte_buf(Vec::new()),
            SectionSlot::Text(text) if text == EMPTY_STRING_SECTION => {
                visitor.visit_byte_buf(Vec::new())
            }
            SectionSlot::Text(text) => visitor.visit_byte_buf(text.into_bytes()),
        }
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.slot {
            SectionSlot::Absent | SectionSlot::Empty => visitor.visit_none(),
            SectionSlot::Text(_) => visitor.visit_some(self),
        }
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_unit()
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char unit unit_struct
        seq tuple tuple_struct map struct enum newtype_struct identifier
    }
}

struct JsonDe(Value);

fn visit_number<'de, V: Visitor<'de>>(
    n: serde_json::Number,
    visitor: V,
) -> Result<V::Value, Error> {
    if let Some(i) = n.as_i64() {
        visitor.visit_i64(i)
    } else if let Some(u) = n.as_u64() {
        visitor.visit_u64(u)
    } else if let Some(f) = n.as_f64() {
        visitor.visit_f64(f)
    } else {
        Err(Error::type_error(format!("invalid number: {n}")))
    }
}

impl<'de> Deserializer<'de> for JsonDe {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.0 {
            Value::Null => visitor.visit_unit(),
            Value::Bool(b) => visitor.visit_bool(b),
            Value::Number(n) => visit_number(n, visitor),
            Value::String(s) => visitor.visit_string(s),
            Value::Array(arr) => visitor.visit_seq(JsonSeq {
                iter: arr.into_iter(),
            }),
            Value::Object(map) => visitor.visit_map(JsonMap {
                iter: map.into_iter(),
                next_value: None,
            }),
        }
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.0 {
            Value::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_unit()
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.0 {
            Value::Null => visitor.visit_unit(),
            other => JsonDe(other).deserialize_any(visitor),
        }
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        visitor.visit_newtype_struct(self)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit_struct seq tuple tuple_struct map struct enum
        identifier
    }
}

struct JsonSeq {
    iter: vec::IntoIter<Value>,
}

impl<'de> SeqAccess<'de> for JsonSeq {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(JsonDe(value)).map(Some),
            None => Ok(None),
        }
    }
}

struct JsonMap {
    iter: <Map<String, Value> as IntoIterator>::IntoIter,
    next_value: Option<Value>,
}

impl<'de> MapAccess<'de> for JsonMap {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.next_value = Some(value);
                seed.deserialize(StringDeserializer::<Error>::new(key))
                    .map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
    where
        V: DeserializeSeed<'de>,
    {
        let value = self
            .next_value
            .take()
            .ok_or_else(|| Error::type_error("value is missing"))?;
        seed.deserialize(JsonDe(value))
    }
}

#[cfg(test)]
mod tests {
    use super::{from_str, section_text};
    use crate::error::ErrorKind;
    use crate::testdata::{goldens, types, values};

    #[test]
    fn page_fenced_yaml_golden() {
        let page: types::Page = from_str(goldens::PAGE_FENCED_YAML).expect("yaml");
        assert_eq!(page, values::page());
    }

    #[cfg(feature = "json")]
    #[test]
    fn page_fenced_json_golden() {
        let page: types::Page = from_str(goldens::PAGE_FENCED_JSON).expect("json");
        assert_eq!(page, values::page());
    }

    #[cfg(feature = "toml")]
    #[test]
    fn page_fenced_toml_golden() {
        let page: types::Page = from_str(goldens::PAGE_FENCED_TOML).expect("toml");
        assert_eq!(page, values::page());
    }

    #[test]
    fn page_fenced_yml_tag() {
        let page: types::Page = from_str(goldens::PAGE_FENCED_YML).expect("yml");
        assert_eq!(page, values::page());
    }

    #[test]
    fn fields_only_fenced_yaml_golden() {
        let fields: types::FieldsOnly =
            from_str(goldens::FIELDS_ONLY_FENCED_YAML).expect("fields-only");
        assert_eq!(fields, values::fields_only());
    }

    #[test]
    fn page_bare_yaml_golden() {
        let yaml: types::Page = from_str(goldens::PAGE_BARE_YAML).expect("bare yaml");
        assert_eq!(yaml, values::page());
    }

    #[cfg(feature = "json")]
    #[test]
    fn page_bare_json_golden() {
        let json: types::Page = from_str(goldens::PAGE_BARE_JSON).expect("bare json");
        assert_eq!(json, values::page());
    }

    #[cfg(feature = "toml")]
    #[test]
    fn page_bare_toml_golden() {
        let toml: types::Page = from_str(goldens::PAGE_BARE_TOML).expect("bare toml");
        assert_eq!(toml, values::page());
    }

    #[test]
    fn page_unlabeled_yaml_sniffs() {
        let yaml: types::Page = from_str(goldens::PAGE_UNLABELED_YAML).expect("unlabeled yaml");
        assert_eq!(yaml, values::page());
    }

    #[cfg(feature = "json")]
    #[test]
    fn page_unlabeled_json_sniffs() {
        let json: types::Page = from_str(goldens::PAGE_UNLABELED_JSON).expect("unlabeled json");
        assert_eq!(json, values::page());
    }

    #[cfg(feature = "toml")]
    #[test]
    fn page_unlabeled_toml_sniffs() {
        let toml: types::Page = from_str(goldens::PAGE_UNLABELED_TOML).expect("unlabeled toml");
        assert_eq!(toml, values::page());
    }

    #[cfg(feature = "json")]
    #[test]
    fn sniffed_json_parse_failure_is_front_matter() {
        // Invalid JSON (unquoted keys) that YAML would accept as a flow mapping.
        // Sniff sees `{` and must not fall back to YAML.
        let unlabeled = "```\n{\n  name: only,\n  count: 2\n}\n```\n";
        let err = from_str::<types::FieldsOnly>(unlabeled).expect_err("unlabeled json");
        assert_eq!(err.kind(), ErrorKind::FrontMatter);
        assert!(std::error::Error::source(&err).is_some());

        let bare = "{\n  name: only,\n  count: 2\n}\n";
        let err = from_str::<types::FieldsOnly>(bare).expect_err("bare json");
        assert_eq!(err.kind(), ErrorKind::FrontMatter);
        assert!(std::error::Error::source(&err).is_some());
    }

    #[cfg(feature = "toml")]
    #[test]
    fn sniffed_toml_parse_failure_is_front_matter() {
        let unlabeled = "```\nname = \"only\"\ncount =\n```\n";
        let err = from_str::<types::FieldsOnly>(unlabeled).expect_err("unlabeled toml");
        assert_eq!(err.kind(), ErrorKind::FrontMatter);
        assert!(std::error::Error::source(&err).is_some());

        let bare = "name = \"only\"\ncount =\n";
        let err = from_str::<types::FieldsOnly>(bare).expect_err("bare toml");
        assert_eq!(err.kind(), ErrorKind::FrontMatter);
        assert!(std::error::Error::source(&err).is_some());
    }

    #[test]
    fn leading_prefix_is_ignored() {
        let page: types::Page = from_str(goldens::PAGE_LEADING_PREFIX).expect("prefix");
        assert_eq!(page, values::page());
    }

    #[test]
    fn body_only_two_sections_golden() {
        let body: types::BodyOnly = from_str(goldens::BODY_ONLY_TWO_SECTIONS).expect("body-only");
        assert_eq!(body, values::body_only());
    }

    #[test]
    fn fence_not_first_is_body_not_fields() {
        let err = from_str::<types::Page>(goldens::PAGE_FENCE_NOT_FIRST).expect_err("not fields");
        assert_eq!(err.kind(), ErrorKind::Body);
        let err = from_str::<types::FieldsOnly>(goldens::PAGE_FENCE_NOT_FIRST)
            .expect_err("fence is not fields");
        assert_eq!(err.kind(), ErrorKind::Body);
    }

    #[test]
    fn page_split_goldens_are_one_first_section_plus_appendix() {
        let fence: types::Page = from_str(goldens::PAGE_SPLIT_FENCE).expect("split fence");
        assert_eq!(fence.field1, "foo");
        assert_eq!(fence.field2, "bar");
        assert_eq!(fence.field3, 1);
        assert_eq!(fence.published, None);
        assert_eq!(
            fence.text1,
            "See this fence; the dashes inside must not split sections:\n\n```text\n---\nnot a split\n```"
        );
        assert_eq!(fence.appendix, values::TEXT2);

        let stars: types::Page = from_str(goldens::PAGE_SPLIT_STARS).expect("split stars");
        assert_eq!(stars.field1, "foo");
        assert_eq!(stars.appendix, values::TEXT2);
        assert_eq!(stars.text1, "before\n***\nstill text1\n___\nstill text1");

        let list: types::Page = from_str(goldens::PAGE_SPLIT_LIST).expect("split list");
        assert_eq!(list.field1, "foo");
        assert_eq!(list.appendix, values::TEXT2);
        assert_eq!(list.text1, "- keep going\n  ---\n- still the same section");
    }

    #[cfg(not(feature = "json"))]
    #[test]
    fn fenced_json_without_feature_is_format_disabled() {
        let err = from_str::<types::Page>(goldens::PAGE_FENCED_JSON).expect_err("json");
        assert_eq!(err.kind(), ErrorKind::FormatDisabled);
        assert!(std::error::Error::source(&err).is_none());
    }

    #[cfg(not(feature = "toml"))]
    #[test]
    fn fenced_toml_without_feature_is_format_disabled() {
        let err = from_str::<types::Page>(goldens::PAGE_FENCED_TOML).expect_err("toml");
        assert_eq!(err.kind(), ErrorKind::FormatDisabled);
        assert!(std::error::Error::source(&err).is_none());
    }

    #[cfg(not(feature = "yaml"))]
    #[test]
    fn fenced_yaml_without_feature_is_format_disabled() {
        let err = from_str::<types::Page>(goldens::PAGE_FENCED_YAML).expect_err("yaml");
        assert_eq!(err.kind(), ErrorKind::FormatDisabled);
        assert!(std::error::Error::source(&err).is_none());
    }

    #[test]
    fn optional_middle_none_golden() {
        let got: types::OptionalBody =
            from_str(goldens::OPTIONAL_MIDDLE_NONE).expect("middle none");
        assert_eq!(got, values::optional_middle_none());
    }

    #[test]
    fn optional_trailing_none_golden() {
        let got: types::OptionalBody =
            from_str(goldens::OPTIONAL_TRAILING_NONE).expect("trailing none");
        assert_eq!(got, values::optional_trailing_none());
    }

    #[test]
    fn optional_leading_none_golden() {
        let got: types::OptionalBody =
            from_str(goldens::OPTIONAL_LEADING_NONE).expect("leading none");
        assert_eq!(got, values::optional_leading_none());
    }

    #[test]
    fn optional_empty_string_golden() {
        let got: types::OptionalBody =
            from_str(goldens::OPTIONAL_EMPTY_STRING).expect("empty string");
        assert_eq!(got, values::optional_empty_string());
    }

    #[test]
    fn extra_optional_body_section_is_body() {
        let input = concat!(
            "```yaml\n",
            "title: memo\n",
            "```\n",
            "a\n",
            "---\n",
            "\n",
            "---\n",
            "c\n",
            "---\n",
            "extra\n"
        );
        let err = from_str::<types::OptionalBody>(input).expect_err("extra section");
        assert_eq!(err.kind(), ErrorKind::Body);
    }

    #[test]
    fn empty_section_for_required_structured_is_body() {
        let input = "```yaml\ntitle: t\n```\n---\n";
        let err = from_str::<types::Article>(input).expect_err("empty structured");
        assert_eq!(err.kind(), ErrorKind::Body);
        assert!(std::error::Error::source(&err).is_none());
    }

    #[test]
    fn structured_decoder_failure_is_body_with_source() {
        let input = "```yaml\ntitle: Hello\n```\n:\n";
        let err = from_str::<types::Article>(input).expect_err("bad structured yaml");
        assert_eq!(err.kind(), ErrorKind::Body);
        assert!(std::error::Error::source(&err).is_some());
    }

    #[test]
    fn nested_fenced_yaml_golden() {
        let got: types::NestedFields = from_str(goldens::NESTED_FENCED_YAML).expect("nested");
        assert_eq!(got, values::nested());
    }

    #[test]
    fn structured_fenced_yaml_golden() {
        let got: types::Article = from_str(goldens::STRUCTURED_FENCED_YAML).expect("structured");
        assert_eq!(got, values::article());
    }

    #[test]
    fn names_fenced_yaml_golden() {
        let got: types::JsonNames = from_str(goldens::NAMES_FENCED_YAML).expect("names");
        assert_eq!(got, values::json_names());
    }

    #[test]
    fn whitespace_unicode_golden_keeps_interiors() {
        let got: types::Page = from_str(goldens::WHITESPACE_UNICODE).expect("whitespace");
        assert_eq!(got, values::whitespace_page());
        assert!(got.text1.contains("spaces   \n"), "{:?}", got.text1);
        assert!(got.text1.contains("🦀"), "{:?}", got.text1);
        assert!(got.text1.contains("Ещё"), "{:?}", got.text1);
    }

    #[test]
    fn extra_body_section_is_body() {
        let input = concat!(
            "```yaml\n",
            "field1: foo\n",
            "field2: bar\n",
            "field3: 1\n",
            "```\n",
            "Text1 bla bla bla\n",
            "---\n",
            "Text2 bal bla bla\n",
            "---\n",
            "extra\n"
        );
        let err = from_str::<types::Page>(input).expect_err("extra section");
        assert_eq!(err.kind(), ErrorKind::Body);
    }

    #[test]
    fn missing_body_section_is_body() {
        let input = concat!(
            "```yaml\n",
            "field1: foo\n",
            "field2: bar\n",
            "field3: 1\n",
            "```\n",
            "only one section\n"
        );
        let err = from_str::<types::Page>(input).expect_err("missing section");
        assert_eq!(err.kind(), ErrorKind::Body);
    }

    #[test]
    fn invalid_fields_yaml_is_front_matter() {
        let input = "```yaml\n:\n```\n";
        let err = from_str::<types::FieldsOnly>(input).expect_err("bad yaml");
        assert_eq!(err.kind(), ErrorKind::FrontMatter);
        assert!(std::error::Error::source(&err).is_some());
    }

    #[test]
    fn section_text_keeps_trailing_spaces() {
        assert_eq!(section_text("hello   \n"), "hello   ");
        assert_eq!(section_text("hello"), "hello");
        assert_eq!(section_text("a\n\nb\n"), "a\n\nb");
    }
}
