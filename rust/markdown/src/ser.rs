//! Serialize a `Markdown` root struct to a document.

use std::cell::Cell;
use std::collections::HashMap;
use std::fmt;
use std::io::Write;

use serde::ser::{Impossible, Serialize, SerializeSeq, SerializeStruct, Serializer};
use serde_json::Value;

use crate::error::Error;
use crate::format::{self, FieldsLayout, Format};
use crate::markdown::Markdown;
use crate::parse::{SEPARATOR, check_middle_section, starts_with_dash_rule, would_read_fields};

/// Serialize `value` as a fenced YAML Markdown document.
///
/// The fields block is a labeled `yaml` fence; its keys follow the order in
/// which the value's `Serialize` impl emits them (declaration order for a
/// derived struct, proto field order for a buffa message). Body `String`
/// fields and types whose serde is a JSON string (well-known `Timestamp`,
/// `Duration`, `FieldMask`) are written as raw section text; nested objects
/// and scalars use the fence-format dump.
///
/// Body sections are joined with `\n---\n`. Text is verbatim: the crate adds
/// no newline to a section and removes none, so the document ends exactly
/// where the last body section ends (there is no newline added at EOF) and a
/// body section that ends with a newline keeps it. The last body field is the
/// rest of the document, so it may hold `---` lines, code fences, tables, and
/// setext underlines. A body field that is not the last is split by a reader
/// on top-level `---`, so such a value, one that contains a top-level `---`
/// line, an unclosed code fence, or ends with a carriage return, is
/// [`crate::ErrorKind::Body`] at write time.
///
/// Trailing `None` body fields are omitted (the last body field has no
/// separator, so it reads back as absent). A `None` that is not trailing is an
/// empty section. For the last body field `Some("")` is its separator followed
/// by nothing, and `Some("\"\"")` is literal text. For a body field that is not
/// the last, an optional `Some("")` is the two characters `""`, so an optional
/// `Some("\"\"")` there is [`crate::ErrorKind::Body`]. A single body field after a
/// fenced block, or in a document with no fields, is written as is, with one
/// leading `---` line when its value is empty or starts with a `---` line. A
/// document with no fields whose body a reader would take for fields starts with
/// an explicit empty fields block.
///
/// [`to_vec`] is these UTF-8 bytes. [`to_writer`] writes the same document.
pub fn to_string<T: Serialize + Markdown>(value: &T) -> Result<String, Error> {
    to_string_with(value, Format::Yaml, FieldsLayout::Fenced)
}

/// Serialize `value` as a fenced Markdown document in `format`.
///
/// Same layout as [`to_string`]: a labeled fence (`yaml`, `json`, or `toml`)
/// then body sections. JSON is pretty-printed with 2-space indent; TOML uses
/// that crate's pretty printer. Keys keep the order of the value's `Serialize`
/// impl in every format; TOML writes plain values before tables, as TOML
/// requires.
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
        format,
    })?;
    let front = if captured.front_len > 0 {
        let front = FrontMatter {
            value,
            body_fields: T::BODY_FIELDS,
            len: captured.front_len,
        };
        Some(format::dump(&front, format)?)
    } else {
        None
    };
    render(front, captured.body, T::BODY_FIELDS, format, layout)
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
    /// Number of root fields that are not body fields.
    front_len: usize,
    body: HashMap<String, BodyValue>,
}

enum BodyValue {
    /// Absent: JSON `null`, `None`, an unset message.
    Null,
    /// Text a reader gets back as this value: a string, or raw UTF-8 from
    /// `serialize_bytes` / `serialize_byte_buf` / a non-empty seq of `u8`.
    Text {
        text: String,
        /// The value was serialized through `serialize_some`.
        optional: bool,
    },
    /// A scalar, object, or array written in the fence language.
    Dumped(String),
}

struct RootSerializer {
    body_fields: &'static [&'static str],
    format: Format,
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
            format: self.format,
            front_len: 0,
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
    format: Format,
    front_len: usize,
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
            let captured = capture_body_field(value, self.format)?;
            self.body.insert(key.to_owned(), captured);
        } else {
            // Front-matter fields are written later, straight from the value,
            // so that the encoder sees them in the order they were declared.
            self.front_len += 1;
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Captured {
            front_len: self.front_len,
            body: self.body,
        })
    }
}

/// The root value without its body fields, serialized in the order the value
/// emits them.
struct FrontMatter<'a, T: ?Sized> {
    value: &'a T,
    body_fields: &'static [&'static str],
    len: usize,
}

impl<T: Serialize + ?Sized> Serialize for FrontMatter<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.value.serialize(FrontOnly {
            inner: serializer,
            body_fields: self.body_fields,
            len: self.len,
        })
    }
}

/// Forwards the root struct to `inner` and drops its body fields.
struct FrontOnly<S> {
    inner: S,
    body_fields: &'static [&'static str],
    len: usize,
}

struct FrontStruct<S> {
    inner: S,
    body_fields: &'static [&'static str],
}

impl<S: SerializeStruct> SerializeStruct for FrontStruct<S> {
    type Ok = S::Ok;
    type Error = S::Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        if self.body_fields.contains(&key) {
            return Ok(());
        }
        self.inner.serialize_field(key, value)
    }

    fn skip_field(&mut self, key: &'static str) -> Result<(), Self::Error> {
        self.inner.skip_field(key)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
    }
}

macro_rules! front_denies {
    ($($method:ident($ty:ty)),* $(,)?) => {
        $(
            fn $method(self, _v: $ty) -> Result<Self::Ok, Self::Error> {
                Err(front_not_struct())
            }
        )*
    };
}

fn front_not_struct<E: serde::ser::Error>() -> E {
    E::custom("root must be a named struct")
}

impl<S: Serializer> Serializer for FrontOnly<S> {
    type Ok = S::Ok;
    type Error = S::Error;
    type SerializeSeq = Impossible<S::Ok, S::Error>;
    type SerializeTuple = Impossible<S::Ok, S::Error>;
    type SerializeTupleStruct = Impossible<S::Ok, S::Error>;
    type SerializeTupleVariant = Impossible<S::Ok, S::Error>;
    type SerializeMap = Impossible<S::Ok, S::Error>;
    type SerializeStruct = FrontStruct<S::SerializeStruct>;
    type SerializeStructVariant = Impossible<S::Ok, S::Error>;

    front_denies! {
        serialize_bool(bool),
        serialize_i8(i8),
        serialize_i16(i16),
        serialize_i32(i32),
        serialize_i64(i64),
        serialize_u8(u8),
        serialize_u16(u16),
        serialize_u32(u32),
        serialize_u64(u64),
        serialize_f32(f32),
        serialize_f64(f64),
        serialize_char(char),
        serialize_str(&str),
        serialize_bytes(&[u8]),
        serialize_unit_struct(&'static str),
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(front_not_struct())
    }

    fn serialize_some<T: ?Sized + Serialize>(self, _value: &T) -> Result<Self::Ok, Self::Error> {
        Err(front_not_struct())
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(front_not_struct())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(front_not_struct())
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        Err(front_not_struct())
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        Err(front_not_struct())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(front_not_struct())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(front_not_struct())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(front_not_struct())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(front_not_struct())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(front_not_struct())
    }

    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        let inner = self.inner.serialize_struct(name, self.len)?;
        Ok(FrontStruct {
            inner,
            body_fields: self.body_fields,
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(front_not_struct())
    }

    fn is_human_readable(&self) -> bool {
        self.inner.is_human_readable()
    }
}

/// Assemble the document.
///
/// `front` is the encoded fields block, if any. Body text is written as is.
/// The separator between two body sections is `\n---\n`; the line break of
/// the closing fence (or of the bare fields' last line) also ends an empty
/// first section.
fn render(
    front: Option<String>,
    mut body: HashMap<String, BodyValue>,
    body_fields: &'static [&'static str],
    format: Format,
    layout: FieldsLayout,
) -> Result<String, Error> {
    let count = body_fields.len();
    let mut slots: Vec<BodyValue> = body_fields
        .iter()
        .map(|name| body.remove(*name).unwrap_or(BodyValue::Null))
        .collect();
    while slots
        .pop_if(|slot| matches!(slot, BodyValue::Null))
        .is_some()
    {}

    let mut texts = Vec::with_capacity(slots.len());
    for (i, slot) in slots.into_iter().enumerate() {
        texts.push(section_text(slot, i + 1 == count)?);
    }

    // After a fence or a bare mapping the document already ends in a line
    // break, which serves as the break that ends an empty first section.
    let after_fields = front.is_some();
    let separated = after_fields && layout == FieldsLayout::Bare;
    let mut region = String::new();
    if count == 1 {
        if let Some(text) = texts.first() {
            if !separated && (text.is_empty() || starts_with_dash_rule(text)) {
                region.push_str("---\n");
            }
            region.push_str(text);
        }
    } else {
        for (i, text) in texts.iter().enumerate() {
            if i == 1 && after_fields && texts[0].is_empty() {
                region.push_str("---\n");
            } else if i > 0 {
                region.push_str(SEPARATOR);
            }
            region.push_str(text);
        }
        // An empty section at the end of the document is no section: close it.
        if texts.len() < count
            && let Some(last) = texts.last()
            && last.is_empty()
        {
            region.push_str(if texts.len() == 1 && after_fields {
                "---\n"
            } else {
                SEPARATOR
            });
        }
    }

    let mut out = String::new();
    match front {
        Some(dumped) => {
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
            if separated && !region.is_empty() {
                out.push_str("---\n");
            }
        }
        None => {
            if !region.is_empty() && would_read_fields(&region, count)? {
                out.push_str(empty_fields_block(format));
            }
        }
    }
    out.push_str(&region);
    Ok(out)
}

/// An explicit fields block with no fields, written before a body-only
/// document whose first text a reader would take for fields.
fn empty_fields_block(format: Format) -> &'static str {
    match format {
        Format::Yaml => "```yaml\n{}\n```\n",
        Format::Json => "```json\n{}\n```\n",
        Format::Toml => "```toml\n```\n",
    }
}

/// The text a reader gets back for this body value, as it goes between
/// separators. `last` is true for the last declared body field.
fn section_text(value: BodyValue, last: bool) -> Result<String, Error> {
    let text = match value {
        BodyValue::Null => return Ok(String::new()),
        BodyValue::Dumped(dumped) if last => return Ok(dumped),
        // The separator that follows ends the encoder's final line.
        BodyValue::Dumped(mut dumped) => {
            if dumped.ends_with('\n') {
                dumped.pop();
            }
            dumped
        }
        BodyValue::Text { text, .. } if last => return Ok(text),
        BodyValue::Text { text, optional } => {
            if text.is_empty() {
                return Ok(if optional {
                    EMPTY_STRING_SECTION.to_owned()
                } else {
                    String::new()
                });
            }
            if optional && text == EMPTY_STRING_SECTION {
                return Err(Error::body(
                    "an optional body value that is not the last cannot be the two \
                     characters \"\"; they stand for an empty value there",
                ));
            }
            text
        }
    };
    check_middle_section(&text)?;
    Ok(text)
}

/// Two characters that stand for an optional empty string in a body section
/// that is not the last (an empty section there is `None`).
const EMPTY_STRING_SECTION: &str = "\"\"";

fn capture_body_field<T: ?Sized + Serialize>(
    value: &T,
    format: Format,
) -> Result<BodyValue, Error> {
    let some = Cell::new(false);
    match value.serialize(BytesProbe { some: &some }) {
        Ok(text) => Ok(BodyValue::Text {
            text,
            optional: some.get(),
        }),
        Err(ProbeError::NotBytes) => {
            let json = serde_json::to_value(value).map_err(Error::type_error)?;
            match json {
                Value::Null => Ok(BodyValue::Null),
                Value::String(text) => Ok(BodyValue::Text {
                    text,
                    optional: some.get(),
                }),
                _ => Ok(BodyValue::Dumped(format::dump(value, format)?)),
            }
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

/// Recognizes a byte body field, and notes whether the value came through
/// `serialize_some` (an optional field that is set).
struct BytesProbe<'a> {
    some: &'a Cell<bool>,
}

fn not_bytes<T>() -> Result<T, ProbeError> {
    Err(ProbeError::NotBytes)
}

impl Serializer for BytesProbe<'_> {
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
        self.some.set(true);
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
        assert!(
            md.ends_with("```\nText1 bla bla bla\n---\nText2 bal bla bla"),
            "{md}"
        );
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
        assert!(
            md.ends_with("---\nText1 bla bla bla\n---\nText2 bal bla bla"),
            "{md}"
        );
        assert!(md.contains("field1: foo"), "{md}");
        assert!(md.contains("field2: bar"), "{md}");
        assert!(md.contains("field3: 1"), "{md}");
        let trimmed = md.trim_end();
        assert!(!trimmed.ends_with("---"), "{md}");
        let back: types::Page = crate::from_str(&md).expect("deserialize");
        assert_eq!(values::page(), back);
    }

    /// Bare YAML whose first field is a list and whose later field holds a
    /// multi-line string with a fenced block: the shape a Knowqore Lesson has.
    /// CommonMark reads the fields as one list item, so the fence sits inside
    /// that item, indented past three spaces on the raw line.
    #[cfg(feature = "yaml")]
    #[test]
    fn bare_yaml_with_a_fenced_string_after_a_list_reads_back() {
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize)]
        struct Artifact {
            path: String,
        }
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize)]
        struct Reason {
            text: String,
        }
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize, crate::Markdown)]
        struct Note {
            artifacts: Vec<Artifact>,
            reason: Reason,
            #[markdown(body)]
            claim: String,
        }
        let note = Note {
            artifacts: vec![Artifact {
                path: "rules/code-first.md".into(),
            }],
            reason: Reason {
                text: "Behaviour ships first.\n\n```text\n# BAD\n# GOOD\n```".into(),
            },
            claim: "Keep one pass.".into(),
        };
        let md = to_string_with(&note, Format::Yaml, FieldsLayout::Bare).expect("serialize");
        assert!(md.contains("```text"), "{md}");
        let back: Note = crate::from_str(&md).expect("deserialize");
        assert_eq!(note, back);
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
        assert!(md.ends_with("```\na\n---\n\n---\nc"), "{md}");
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn optional_trailing_none_omits_separator() {
        let value = values::optional_trailing_none();
        assert_optional_round_trip(&value, goldens::OPTIONAL_TRAILING_NONE);
        let md = to_string(&value).expect("serialize");
        assert!(md.ends_with("```\na"), "{md}");
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
        assert!(md.ends_with("```\n---\na"), "{md}");
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn optional_empty_string_writes_quotes() {
        let value = values::optional_empty_string();
        assert_optional_round_trip(&value, goldens::OPTIONAL_EMPTY_STRING);
        let md = to_string(&value).expect("serialize");
        assert!(md.ends_with("```\na\n---\n\"\""), "{md}");
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
            md.ends_with("---\n1.5s") || md.ends_with("---\n1.500s"),
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
            md.ends_with("```\ncafé"),
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
        assert!(md.ends_with("```\nhello"), "{md}");
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
