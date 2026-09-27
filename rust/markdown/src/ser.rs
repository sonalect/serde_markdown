//! Serialize a `Markdown` root struct to a document.

use std::collections::HashMap;
use std::fmt;
use std::io::Write;

use serde::ser::{Impossible, Serialize, SerializeSeq, SerializeStruct, Serializer};
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
///
/// [`to_vec`] is these UTF-8 bytes. [`to_writer`] writes the same document.
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

/// Serialize `value` as UTF-8 bytes of a fenced YAML Markdown document.
///
/// Same document as [`to_string`].
pub fn to_vec<T: Serialize + Markdown>(value: &T) -> Result<Vec<u8>, Error> {
    to_string(value).map(String::into_bytes)
}

/// Write `value` as a fenced YAML Markdown document to `writer`.
///
/// Same document as [`to_string`]. A write failure is [`crate::ErrorKind::Io`]
/// with `source` set to [`std::io::Error`].
pub fn to_writer<W, T>(mut writer: W, value: &T) -> Result<(), Error>
where
    W: Write,
    T: Serialize + Markdown,
{
    writer.write_all(to_string(value)?.as_bytes())?;
    Ok(())
}

struct Captured {
    fields: Map<String, Value>,
    body: HashMap<String, BodyValue>,
}

enum BodyValue {
    Json(Value),
    /// Raw UTF-8 from `serialize_bytes` / `serialize_byte_buf` / a non-empty
    /// seq of `u8`. Empty JSON strings still use the `""` encoding; this
    /// variant is written as-is, including empty.
    Utf8(String),
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
            body: HashMap::new(),
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
    body: HashMap<String, BodyValue>,
}

impl SerializeStruct for StructSerializer {
    type Ok = Captured;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        if self.body_fields.contains(&key) {
            self.body.insert(key.to_owned(), capture_body_field(value)?);
        } else {
            let json = serde_json::to_value(value).map_err(Error::type_error)?;
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

fn body_slot(value: Option<&BodyValue>, format: Format) -> Result<BodySlot, Error> {
    match value {
        None | Some(BodyValue::Json(Value::Null)) => Ok(BodySlot::Absent),
        Some(BodyValue::Utf8(s)) => Ok(BodySlot::Raw(s.clone())),
        Some(BodyValue::Json(Value::String(s))) if s.is_empty() => {
            Ok(BodySlot::Raw("\"\"".to_owned()))
        }
        Some(BodyValue::Json(Value::String(s))) => Ok(BodySlot::Raw(s.clone())),
        Some(BodyValue::Json(other)) => Ok(BodySlot::Raw(format::dump(other, format)?)),
    }
}

fn capture_body_field<T: ?Sized + Serialize>(value: &T) -> Result<BodyValue, Error> {
    match value.serialize(BytesProbe) {
        Ok(text) => Ok(BodyValue::Utf8(text)),
        Err(ProbeError::NotBytes) => {
            let json = serde_json::to_value(value).map_err(Error::type_error)?;
            Ok(BodyValue::Json(json))
        }
        Err(ProbeError::InvalidUtf8) => Err(Error::body("invalid UTF-8 in byte body field")),
        Err(ProbeError::Other(err)) => Err(err),
    }
}

enum ProbeError {
    NotBytes,
    InvalidUtf8,
    Other(Error),
}

impl fmt::Debug for ProbeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotBytes => f.write_str("NotBytes"),
            Self::InvalidUtf8 => f.write_str("InvalidUtf8"),
            Self::Other(err) => fmt::Debug::fmt(err, f),
        }
    }
}

impl fmt::Display for ProbeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotBytes => f.write_str("not a byte body field"),
            Self::InvalidUtf8 => f.write_str("invalid UTF-8 in byte body field"),
            Self::Other(err) => fmt::Display::fmt(err, f),
        }
    }
}

impl std::error::Error for ProbeError {}

impl serde::ser::Error for ProbeError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self::Other(Error::type_error(msg))
    }
}

struct BytesProbe;

fn not_bytes<T>() -> Result<T, ProbeError> {
    Err(ProbeError::NotBytes)
}

impl Serializer for BytesProbe {
    type Ok = String;
    type Error = ProbeError;
    type SerializeSeq = ProbeSeq;
    type SerializeTuple = Impossible<String, ProbeError>;
    type SerializeTupleStruct = Impossible<String, ProbeError>;
    type SerializeTupleVariant = Impossible<String, ProbeError>;
    type SerializeMap = Impossible<String, ProbeError>;
    type SerializeStruct = Impossible<String, ProbeError>;
    type SerializeStructVariant = Impossible<String, ProbeError>;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        String::from_utf8(v.to_vec()).map_err(|_| ProbeError::InvalidUtf8)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(ProbeSeq {
            buf: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        not_bytes()
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        not_bytes()
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        not_bytes()
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        not_bytes()
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        not_bytes()
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        not_bytes()
    }
}

struct ProbeSeq {
    buf: Vec<u8>,
}

struct U8Element;

impl Serializer for U8Element {
    type Ok = u8;
    type Error = ProbeError;
    type SerializeSeq = Impossible<u8, ProbeError>;
    type SerializeTuple = Impossible<u8, ProbeError>;
    type SerializeTupleStruct = Impossible<u8, ProbeError>;
    type SerializeTupleVariant = Impossible<u8, ProbeError>;
    type SerializeMap = Impossible<u8, ProbeError>;
    type SerializeStruct = Impossible<u8, ProbeError>;
    type SerializeStructVariant = Impossible<u8, ProbeError>;

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(v)
    }

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_some<T: ?Sized + Serialize>(self, _value: &T) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        not_bytes()
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        not_bytes()
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        not_bytes()
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        not_bytes()
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        not_bytes()
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        not_bytes()
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        not_bytes()
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        not_bytes()
    }
}

impl SerializeSeq for ProbeSeq {
    type Ok = String;
    type Error = ProbeError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.buf.push(value.serialize(U8Element)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.buf.is_empty() {
            return Err(ProbeError::NotBytes);
        }
        String::from_utf8(self.buf).map_err(|_| ProbeError::InvalidUtf8)
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

    use super::{to_string, to_string_with, to_string_with_format, to_vec, to_writer};
    use crate::error::ErrorKind;
    use crate::format::{FieldsLayout, Format};
    use crate::markdown::Markdown;
    #[cfg(feature = "yaml")]
    use crate::testdata::goldens;
    use crate::testdata::{types, values};

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

    #[cfg(feature = "yaml")]
    #[test]
    fn page_round_trip() {
        let page = values::page();
        let md = to_string(&page).expect("serialize");
        let back: types::Page = crate::from_str(&md).expect("deserialize");
        assert_eq!(page, back);
    }

    #[cfg(feature = "yaml")]
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

    #[cfg(feature = "yaml")]
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

    #[cfg(feature = "yaml")]
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

    #[cfg(feature = "yaml")]
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

    #[cfg(feature = "yaml")]
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

    #[cfg(feature = "yaml")]
    fn assert_optional_round_trip(value: &types::OptionalBody, golden: &str) {
        let from_golden: types::OptionalBody = crate::from_str(golden).expect("golden");
        assert_eq!(*value, from_golden);
        let md = to_string(value).expect("serialize");
        let back: types::OptionalBody = crate::from_str(&md).expect("round-trip");
        assert_eq!(*value, back);
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn optional_middle_none_keeps_empty_slot() {
        let value = values::optional_middle_none();
        assert_optional_round_trip(&value, goldens::OPTIONAL_MIDDLE_NONE);
        let md = to_string(&value).expect("serialize");
        assert!(md.contains("```\na\n---\n\n---\nc\n"), "{md}");
    }

    #[cfg(feature = "yaml")]
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

    #[cfg(feature = "yaml")]
    #[test]
    fn optional_leading_none_empty_first_section() {
        let value = values::optional_leading_none();
        assert_optional_round_trip(&value, goldens::OPTIONAL_LEADING_NONE);
        let md = to_string(&value).expect("serialize");
        assert!(md.contains("```\n---\na\n"), "{md}");
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn optional_empty_string_writes_quotes() {
        let value = values::optional_empty_string();
        assert_optional_round_trip(&value, goldens::OPTIONAL_EMPTY_STRING);
        let md = to_string(&value).expect("serialize");
        assert!(md.contains("```\na\n---\n\"\"\n"), "{md}");
    }

    #[cfg(feature = "yaml")]
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

    #[cfg(feature = "yaml")]
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

    #[cfg(feature = "yaml")]
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

    #[cfg(feature = "yaml")]
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

    #[cfg(feature = "yaml")]
    #[test]
    fn whitespace_round_trip_keeps_interiors() {
        let value = values::whitespace_page();
        let md = to_string(&value).expect("serialize");
        assert!(md.contains("spaces   \n"), "{md}");
        assert!(md.contains("🦀"), "{md}");
        let back: types::Page = crate::from_str(&md).expect("round-trip");
        assert_eq!(value, back);
    }

    #[cfg(feature = "yaml")]
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

    #[cfg(feature = "yaml")]
    #[test]
    fn well_known_generated_round_trip() {
        let value = values::well_known_generated();
        let from_golden: serde_markdown_proto::markdown::testdata::WellKnown =
            crate::from_str(goldens::WELL_KNOWN_FENCED_YAML).expect("golden");
        assert_eq!(value, from_golden);
        let md = to_string(&value).expect("serialize");
        let back: serde_markdown_proto::markdown::testdata::WellKnown =
            crate::from_str(&md).expect("round-trip");
        assert_eq!(value, back);
    }

    #[cfg(feature = "yaml")]
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

    #[cfg(feature = "yaml")]
    #[test]
    fn page_published_generated_round_trip() {
        let value = values::page_generated_with_published();
        let from_golden: serde_markdown_proto::markdown::testdata::Page =
            crate::from_str(goldens::PAGE_PUBLISHED_YAML).expect("golden");
        assert_eq!(value, from_golden);
        let md = to_string(&value).expect("serialize");
        let back: serde_markdown_proto::markdown::testdata::Page =
            crate::from_str(&md).expect("round-trip");
        assert_eq!(value, back);
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn page_generated_round_trip() {
        let value = values::page_generated();
        let from_golden: serde_markdown_proto::markdown::testdata::Page =
            crate::from_str(goldens::PAGE_FENCED_YAML).expect("golden");
        assert_eq!(value, from_golden);
        let md = to_string(&value).expect("serialize");
        let back: serde_markdown_proto::markdown::testdata::Page =
            crate::from_str(&md).expect("round-trip");
        assert_eq!(value, back);
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn proto3_page_generated_round_trip() {
        let value = values::proto3_page_generated();
        let from_golden: serde_markdown_proto::markdown::testdata::Proto3Page =
            crate::from_str(goldens::PROTO3_FENCED_YAML).expect("golden");
        assert_eq!(value, from_golden);
        let md = to_string(&value).expect("serialize");
        let back: serde_markdown_proto::markdown::testdata::Proto3Page =
            crate::from_str(&md).expect("round-trip");
        assert_eq!(value, back);
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn json_names_generated_round_trip() {
        let value = values::json_names_generated();
        let from_golden: serde_markdown_proto::markdown::testdata::JsonNames =
            crate::from_str(goldens::NAMES_FENCED_YAML).expect("golden");
        assert_eq!(value, from_golden);
        let md = to_string(&value).expect("serialize");
        let back: serde_markdown_proto::markdown::testdata::JsonNames =
            crate::from_str(&md).expect("round-trip");
        assert_eq!(value, back);
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn optional_body_generated_round_trip() {
        let value = values::optional_middle_none_generated();
        let from_golden: serde_markdown_proto::markdown::testdata::OptionalBody =
            crate::from_str(goldens::OPTIONAL_MIDDLE_NONE).expect("golden");
        assert_eq!(value, from_golden);
        let md = to_string(&value).expect("serialize");
        let back: serde_markdown_proto::markdown::testdata::OptionalBody =
            crate::from_str(&md).expect("round-trip");
        assert_eq!(value, back);
        let trailing = values::optional_trailing_none_generated();
        let from_trailing: serde_markdown_proto::markdown::testdata::OptionalBody =
            crate::from_str(goldens::OPTIONAL_TRAILING_NONE).expect("trailing");
        assert_eq!(trailing, from_trailing);
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn article_generated_round_trip() {
        let value = values::article_generated();
        let from_golden: serde_markdown_proto::markdown::testdata::Article =
            crate::from_str(goldens::STRUCTURED_FENCED_YAML).expect("golden");
        assert_eq!(value, from_golden);
        let md = to_string(&value).expect("serialize");
        let back: serde_markdown_proto::markdown::testdata::Article =
            crate::from_str(&md).expect("round-trip");
        assert_eq!(value, back);
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn to_vec_matches_to_string() {
        let md = to_string(&values::page()).expect("serialize");
        let bytes = to_vec(&values::page()).expect("to_vec");
        assert_eq!(bytes, md.as_bytes());
        let back: types::Page = crate::from_slice(&bytes).expect("from_slice");
        assert_eq!(values::page(), back);
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn to_writer_matches_to_string() {
        let md = to_string(&values::page()).expect("serialize");
        let mut buf = Vec::new();
        to_writer(&mut buf, &values::page()).expect("to_writer");
        assert_eq!(buf, md.as_bytes());
        let back: types::Page = crate::from_reader(buf.as_slice()).expect("from_reader");
        assert_eq!(values::page(), back);
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn to_writer_io_error_is_io() {
        struct Boom;
        impl std::io::Write for Boom {
            fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("boom"))
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let err = to_writer(Boom, &values::page()).expect_err("write");
        assert_eq!(err.kind(), ErrorKind::Io);
        assert!(std::error::Error::source(&err).is_some());
    }

    #[cfg(feature = "yaml")]
    #[derive(Debug, PartialEq, serde::Deserialize, Serialize, crate::Markdown)]
    struct BytesDoc {
        title: String,
        #[markdown(body)]
        data: Vec<u8>,
    }

    #[cfg(feature = "yaml")]
    #[derive(Debug, PartialEq)]
    struct ByteField(Vec<u8>);

    #[cfg(feature = "yaml")]
    impl Serialize for ByteField {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_bytes(&self.0)
        }
    }

    #[cfg(feature = "yaml")]
    impl<'de> serde::Deserialize<'de> for ByteField {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            struct Vis;
            impl<'de> serde::de::Visitor<'de> for Vis {
                type Value = ByteField;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("bytes")
                }

                fn visit_bytes<E: serde::de::Error>(self, v: &[u8]) -> Result<ByteField, E> {
                    Ok(ByteField(v.to_vec()))
                }

                fn visit_byte_buf<E: serde::de::Error>(self, v: Vec<u8>) -> Result<ByteField, E> {
                    Ok(ByteField(v))
                }
            }
            deserializer.deserialize_byte_buf(Vis)
        }
    }

    #[cfg(feature = "yaml")]
    #[derive(Debug, PartialEq, serde::Deserialize, Serialize, crate::Markdown)]
    struct ByteBufDoc {
        title: String,
        #[markdown(body)]
        data: ByteField,
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn vec_u8_body_is_raw_utf8() {
        let value = BytesDoc {
            title: "bin".into(),
            data: b"caf\xc3\xa9".to_vec(),
        };
        let md = to_string(&value).expect("serialize");
        assert!(md.contains("```yaml\n"), "{md}");
        assert!(md.contains("title: bin"), "{md}");
        assert!(
            md.contains("```\ncafé\n"),
            "Vec<u8> body must be raw UTF-8, got {md}"
        );
        assert!(
            !md.contains("99") && !md.contains("- 99"),
            "Vec<u8> must not dump as a number array, got {md}"
        );
        let back: BytesDoc = crate::from_str(&md).expect("round-trip");
        assert_eq!(value, back);
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn serialize_bytes_body_is_raw_utf8() {
        let value = ByteBufDoc {
            title: "bin".into(),
            data: ByteField(b"hello".to_vec()),
        };
        let md = to_string(&value).expect("serialize");
        assert!(md.contains("```\nhello\n"), "{md}");
        let back: ByteBufDoc = crate::from_str(&md).expect("round-trip");
        assert_eq!(value, back);
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn invalid_utf8_byte_body_is_body() {
        let value = BytesDoc {
            title: "bin".into(),
            data: vec![0xff, b'a'],
        };
        let err = to_string(&value).expect_err("utf8");
        assert_eq!(err.kind(), ErrorKind::Body);
        assert!(std::error::Error::source(&err).is_none());
        assert!(err.to_string().contains("UTF-8"), "{err}");

        let via_bytes = ByteBufDoc {
            title: "bin".into(),
            data: ByteField(vec![0xff]),
        };
        let err = to_string(&via_bytes).expect_err("serialize_bytes utf8");
        assert_eq!(err.kind(), ErrorKind::Body);
    }

    #[cfg(not(feature = "yaml"))]
    #[test]
    fn yaml_without_feature_is_format_disabled() {
        let err = to_string(&values::page()).expect_err("disabled");
        assert_eq!(err.kind(), ErrorKind::FormatDisabled);
        assert!(std::error::Error::source(&err).is_none());
        let err = to_vec(&values::page()).expect_err("to_vec");
        assert_eq!(err.kind(), ErrorKind::FormatDisabled);
        let err = to_writer(std::io::sink(), &values::page()).expect_err("to_writer");
        assert_eq!(err.kind(), ErrorKind::FormatDisabled);
    }
}
