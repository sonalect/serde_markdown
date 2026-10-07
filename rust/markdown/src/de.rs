//! Deserialize a `Markdown` root struct from a document.

use std::cell::Cell;
use std::io::Read;
use std::marker::PhantomData;
use std::vec;

use serde::de::value::{StrDeserializer, StringDeserializer};
use serde::de::{
    DeserializeOwned, DeserializeSeed, Deserializer, EnumAccess, MapAccess, SeqAccess,
    VariantAccess, Visitor,
};
use serde_json::{Map, Value};

use crate::drive::{self, Steps};
use crate::error::{Error, ErrorKind};
use crate::format::{self, Format};
use crate::markdown::Markdown;
use crate::parse::{self, Document, Fields};

/// Deserialize a Markdown document from `s`.
///
/// The optional fields block is the first fenced `yaml` / `yml` / `json` /
/// `toml` code block, or a **bare** mapping before the first top-level `---`.
/// An unlabeled first fence and a bare first slice are sniffed: `{` / `[`
/// means JSON, a TOML-shaped first line means TOML, otherwise YAML. A sniffed
/// JSON or TOML parse failure is [`ErrorKind::FrontMatter`] and does not fall
/// back to YAML. A bare first slice that is not a mapping is the first body
/// section. Leading Unicode whitespace and top-level `---` before fields are
/// discarded only when fields follow; a document with no fields is body from
/// its first byte, nothing skipped. A fenced `yaml` block that is not first
/// is body.
///
/// Body fields listed in [`Markdown::BODY_FIELDS`] are filled from sections
/// in declaration order. Section text is verbatim: of the bytes between two
/// separators only the one line break that ends the section is dropped, and
/// nothing is trimmed. The last body field is every byte after its separator
/// up to the end of the document, never parsed, so it may hold `---` lines,
/// code fences, tables, and setext underlines; the earlier fields end at the
/// first top-level `---` lines. `String` / bytes sections are the raw section
/// text. Types whose proto3 JSON serde is a string (well-known `Timestamp`,
/// `Duration`, `FieldMask`) read that same raw text. Nested structs, maps,
/// sequences, and scalars parse as the fields format when a fields block was
/// present, otherwise YAML.
///
/// Protobuf `Any` packing is not installed here. Callers who need `Any` must
/// install a type registry (`buffa_types::register_wkt_types`) the same way
/// they do for `serde_json`.
///
/// A body field whose section is missing is absent: `None` on an optional
/// field. The last body field is present when its separator is, so a
/// separator followed by nothing is `Some("")`, and the two characters `""`
/// after it are literal text. In a section that is not the last, an empty
/// section is `None` for an optional field and `""` for a required string,
/// and an optional field reads the two characters `""` as `Some("")`. With a
/// single body field after a fenced block, or in a document with no fields,
/// a first line that is a `---` rule is that field's separator and is dropped
/// (once). A missing required section, extra body text when there are no body
/// fields, and an empty section for a required structured field are
/// [`ErrorKind::Body`]. A decoder failure on a structured section is
/// [`ErrorKind::Body`] with `source` set.
///
/// [`from_slice`] is the same mapping on UTF-8 bytes. [`from_reader`] reads
/// a stream to the end, then uses [`from_slice`].
pub fn from_str<T>(s: &str) -> Result<T, Error>
where
    T: DeserializeOwned + Markdown,
{
    drive::run(ReadDoc::new(s))
}

/// Reading a document, one step at a time: split it into the fields slice
/// and body sections, decode the fields block, and deserialize the value.
/// The value's own `Deserialize` runs as one step, so a structured body
/// section is decoded within it.
pub(crate) struct ReadDoc<'a, T> {
    input: &'a str,
    body_fields: &'static [&'static str],
    stage: ReadStage,
    value: PhantomData<fn() -> T>,
}

enum ReadStage {
    Split,
    Decode(Document),
    Deserialize(Decoded),
    Done,
}

impl<'a, T: DeserializeOwned + Markdown> ReadDoc<'a, T> {
    /// The job that reads a `T` from `input`.
    pub(crate) fn new(input: &'a str) -> Self {
        Self {
            input,
            body_fields: T::BODY_FIELDS,
            stage: ReadStage::Split,
            value: PhantomData,
        }
    }
}

impl<T: DeserializeOwned> Steps for ReadDoc<'_, T> {
    type Output = T;

    fn step(&mut self) -> Result<Option<T>, Error> {
        let count = self.body_fields.len();
        match std::mem::replace(&mut self.stage, ReadStage::Done) {
            ReadStage::Split => {
                self.stage = ReadStage::Decode(parse::parse(self.input, count)?);
            }
            ReadStage::Decode(doc) => {
                let decoded = decode_document(doc)?;
                if decoded.body.len() > count {
                    return Err(Error::body(format!(
                        "expected at most {count} body section(s), found {}",
                        decoded.body.len()
                    )));
                }
                self.stage = ReadStage::Deserialize(decoded);
            }
            ReadStage::Deserialize(decoded) => {
                let slots = section_slots(count, decoded.body);
                let finished = Cell::new(false);
                let read = serde::de::Deserialize::deserialize(RootDe {
                    fields: decoded.fields,
                    body_fields: self.body_fields,
                    slots,
                    format: decoded.format,
                    finished: &finished,
                });
                return read.map(Some).map_err(|err| {
                    // A body field the document has no section for is not
                    // handed to the value, so `default` and `Option` apply;
                    // a required one fails in the value's own visitor, after
                    // every key was handed over.
                    match err.missing_field() {
                        Some(name) if finished.get() && self.body_fields.contains(&name) => {
                            Error::body(format!("missing section for body field `{name}`"))
                        }
                        _ => err,
                    }
                });
            }
            ReadStage::Done => unreachable!("a finished job is not stepped again"),
        }
        Ok(None)
    }
}

/// Deserialize a Markdown document from UTF-8 `bytes`.
///
/// Same mapping as [`from_str`]. Invalid UTF-8 is [`crate::ErrorKind::Syntax`] with
/// [`Error::offset`] at the first invalid byte.
pub fn from_slice<T>(bytes: &[u8]) -> Result<T, Error>
where
    T: DeserializeOwned + Markdown,
{
    match str::from_utf8(bytes) {
        Ok(s) => from_str(s),
        Err(err) => Err(Error::syntax(err.valid_up_to(), "invalid UTF-8")),
    }
}

/// Deserialize a Markdown document by reading `reader` to the end.
///
/// Same mapping as [`from_str`] after the bytes are collected. A read failure
/// is [`crate::ErrorKind::Io`] with `source` set to [`std::io::Error`]. Invalid UTF-8
/// is [`crate::ErrorKind::Syntax`], same as [`from_slice`].
pub fn from_reader<R, T>(mut reader: R) -> Result<T, Error>
where
    R: Read,
    T: DeserializeOwned + Markdown,
{
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    from_slice(&bytes)
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
            let format = labeled.unwrap_or(sniff).format();
            let map = mapping_or_type_error(format::load(&inner, format)?)?;
            Ok(Decoded {
                fields: map,
                body,
                format,
            })
        }
        Fields::Bare { sniff, map, .. } => Ok(Decoded {
            fields: map,
            body,
            format: sniff.format(),
        }),
        Fields::None => Ok(Decoded {
            fields: Map::new(),
            body,
            format: Format::Yaml,
        }),
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

/// Two characters that stand for `Some("")` in a section that is not the
/// last (an empty section there is `None`).
const EMPTY_STRING_SECTION: &str = "\"\"";

enum SectionSlot {
    /// The document has no such section.
    Absent,
    /// A section that is not the last and has no text.
    Empty,
    /// A section that is not the last and holds the two characters `""`.
    /// An optional field reads it as `Some("")`; any other type reads the text.
    QuotedEmpty,
    /// Section text. Empty only for a last body field whose separator is
    /// followed by nothing.
    Text(String),
}

fn section_slots(n: usize, sections: Vec<String>) -> Vec<SectionSlot> {
    let mut sections = sections.into_iter();
    (0..n)
        .map(|i| match sections.next() {
            None => SectionSlot::Absent,
            Some(text) if i + 1 == n => SectionSlot::Text(text),
            Some(text) if text.is_empty() => SectionSlot::Empty,
            Some(text) if text == EMPTY_STRING_SECTION => SectionSlot::QuotedEmpty,
            Some(text) => SectionSlot::Text(text),
        })
        .collect()
}

fn into_body(err: Error) -> Error {
    match err.kind() {
        ErrorKind::Body | ErrorKind::FormatDisabled | ErrorKind::Syntax => err,
        _ => Error::body(err),
    }
}

struct RootDe<'a> {
    fields: Map<String, Value>,
    body_fields: &'static [&'static str],
    slots: Vec<SectionSlot>,
    format: Format,
    /// Set once every key was handed to the value.
    finished: &'a Cell<bool>,
}

impl<'de> Deserializer<'de> for RootDe<'_> {
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

struct RootMap<'a> {
    fields: Map<String, Value>,
    slots: Vec<SectionSlot>,
    format: Format,
    pending: vec::IntoIter<Pending>,
    current: Option<Pending>,
    finished: &'a Cell<bool>,
}

impl<'a> RootMap<'a> {
    /// The keys handed to the value: every key of the fields block that is
    /// not a body field, then each body field the document has a section
    /// for. A body field without a section is left out, so the value's own
    /// `default` or `Option` applies.
    fn new(root: RootDe<'a>) -> Self {
        let mut pending = Vec::with_capacity(root.fields.len() + root.body_fields.len());
        for key in root.fields.keys() {
            if !root.body_fields.contains(&key.as_str()) {
                pending.push(Pending::Front(key.clone()));
            }
        }
        for (index, name) in root.body_fields.iter().enumerate() {
            if !matches!(root.slots.get(index), None | Some(SectionSlot::Absent)) {
                pending.push(Pending::Body { name, index });
            }
        }
        Self {
            fields: root.fields,
            slots: root.slots,
            format: root.format,
            pending: pending.into_iter(),
            current: None,
            finished: root.finished,
        }
    }
}

impl<'de> MapAccess<'de> for RootMap<'_> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: DeserializeSeed<'de>,
    {
        match self.pending.next() {
            None => {
                self.finished.set(true);
                Ok(None)
            }
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
                    .get_mut(index)
                    .map_or(SectionSlot::Absent, |slot| {
                        std::mem::replace(slot, SectionSlot::Absent)
                    });
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
            SectionSlot::Text(text) if text.is_empty() => {
                Err(Error::body("empty section for required structured field"))
            }
            SectionSlot::QuotedEmpty => Ok(JsonDe(format::load_body(
                EMPTY_STRING_SECTION,
                self.format,
            )?)),
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
            SectionSlot::QuotedEmpty => visitor.visit_string(EMPTY_STRING_SECTION.to_owned()),
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
            SectionSlot::QuotedEmpty => {
                visitor.visit_byte_buf(EMPTY_STRING_SECTION.as_bytes().to_vec())
            }
            SectionSlot::Text(text) => visitor.visit_byte_buf(text.into_bytes()),
        }
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.slot {
            SectionSlot::Absent | SectionSlot::Empty => visitor.visit_none(),
            SectionSlot::QuotedEmpty => visitor.visit_some(SectionDeserializer {
                slot: SectionSlot::Text(String::new()),
                format: self.format,
            }),
            SectionSlot::Text(_) => visitor.visit_some(self),
        }
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_unit()
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.slot {
            SectionSlot::Absent => Err(Error::body("missing section")),
            SectionSlot::Empty => visitor.visit_seq(JsonSeq {
                iter: Vec::new().into_iter(),
            }),
            SectionSlot::QuotedEmpty => visitor.visit_seq(ByteSeq {
                iter: EMPTY_STRING_SECTION.as_bytes().to_vec().into_iter(),
            }),
            // Text that reads as a sequence is ambiguous: a `Vec<u8>` is
            // written as its raw text, any other sequence in the fence
            // language. The element type decides, at the first element.
            SectionSlot::Text(text) => match format::load_body(&text, self.format) {
                Ok(Value::Array(values)) if values.is_empty() => visitor.visit_seq(JsonSeq {
                    iter: values.into_iter(),
                }),
                Ok(Value::Array(values)) => visitor.visit_seq(EitherSeq::Undecided {
                    bytes: text.into_bytes(),
                    values,
                }),
                _ => visitor.visit_seq(ByteSeq {
                    iter: text.into_bytes().into_iter(),
                }),
            },
        }
    }

    /// A unit variant is written as its raw name; any other variant in the
    /// fence language.
    fn deserialize_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        match self.slot {
            SectionSlot::Text(text) if variants.contains(&text.as_str()) => {
                StringDeserializer::<Error>::new(text).deserialize_enum(name, variants, visitor)
            }
            slot => SectionDeserializer {
                slot,
                format: self.format,
            }
            .parsed()?
            .deserialize_enum(name, variants, visitor)
            .map_err(into_body),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char unit unit_struct
        tuple tuple_struct map struct newtype_struct identifier
    }
}

/// A body section that reads as raw bytes and as a sequence in the fence
/// language; the first element decides which.
enum EitherSeq {
    Undecided { bytes: Vec<u8>, values: Vec<Value> },
    Bytes(vec::IntoIter<u8>),
    Values(vec::IntoIter<Value>),
}

/// How the first element of an [`EitherSeq`] asked to be read.
enum Choice {
    Bytes,
    Values,
}

impl<'de> SeqAccess<'de> for EitherSeq {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self {
            EitherSeq::Bytes(iter) => match iter.next() {
                Some(b) => seed
                    .deserialize(JsonDe(Value::Number(u64::from(b).into())))
                    .map(Some),
                None => Ok(None),
            },
            EitherSeq::Values(iter) => match iter.next() {
                Some(value) => seed.deserialize(JsonDe(value)).map(Some),
                None => Ok(None),
            },
            EitherSeq::Undecided { bytes, values } => {
                let mut bytes = std::mem::take(bytes).into_iter();
                let mut values = std::mem::take(values).into_iter();
                let (Some(byte), Some(value)) = (bytes.next(), values.next()) else {
                    *self = EitherSeq::Values(Vec::new().into_iter());
                    return Ok(None);
                };
                let mut choice = Choice::Values;
                let first = seed.deserialize(FirstElement {
                    byte,
                    value,
                    choice: &mut choice,
                })?;
                *self = match choice {
                    Choice::Bytes => EitherSeq::Bytes(bytes),
                    Choice::Values => EitherSeq::Values(values),
                };
                Ok(Some(first))
            }
        }
    }
}

/// The first element of an [`EitherSeq`]: a `u8` takes the first byte of
/// the text, anything else the first decoded value.
struct FirstElement<'a> {
    byte: u8,
    value: Value,
    choice: &'a mut Choice,
}

macro_rules! first_element_reads_values {
    ($($method:ident),* $(,)?) => {
        $(
            fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
                *self.choice = Choice::Values;
                JsonDe(self.value).$method(visitor)
            }
        )*
    };
}

impl<'de> Deserializer<'de> for FirstElement<'_> {
    type Error = Error;

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        *self.choice = Choice::Bytes;
        visitor.visit_u8(self.byte)
    }

    first_element_reads_values! {
        deserialize_any, deserialize_bool, deserialize_i8, deserialize_i16, deserialize_i32,
        deserialize_i64, deserialize_i128, deserialize_u16, deserialize_u32, deserialize_u64,
        deserialize_u128, deserialize_f32, deserialize_f64, deserialize_char, deserialize_str,
        deserialize_string, deserialize_bytes, deserialize_byte_buf, deserialize_option,
        deserialize_unit, deserialize_seq, deserialize_map, deserialize_identifier,
        deserialize_ignored_any,
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        *self.choice = Choice::Values;
        JsonDe(self.value).deserialize_unit_struct(name, visitor)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        *self.choice = Choice::Values;
        JsonDe(self.value).deserialize_newtype_struct(name, visitor)
    }

    fn deserialize_tuple<V: Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value, Error> {
        *self.choice = Choice::Values;
        JsonDe(self.value).deserialize_tuple(len, visitor)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Error> {
        *self.choice = Choice::Values;
        JsonDe(self.value).deserialize_tuple_struct(name, len, visitor)
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        *self.choice = Choice::Values;
        JsonDe(self.value).deserialize_struct(name, fields, visitor)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        *self.choice = Choice::Values;
        JsonDe(self.value).deserialize_enum(name, variants, visitor)
    }
}

struct ByteSeq {
    iter: vec::IntoIter<u8>,
}

impl<'de> SeqAccess<'de> for ByteSeq {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(b) => seed
                .deserialize(JsonDe(Value::Number(u64::from(b).into())))
                .map(Some),
            None => Ok(None),
        }
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

    /// An enum value is its variant name, or a map from the variant name to
    /// its content: the forms JSON and TOML write, and YAML tags become.
    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        match self.0 {
            Value::String(variant) => visitor.visit_enum(JsonEnum {
                variant,
                content: None,
            }),
            Value::Object(map) => {
                let mut entries = map.into_iter();
                match (entries.next(), entries.next()) {
                    (Some((variant, content)), None) => visitor.visit_enum(JsonEnum {
                        variant,
                        content: Some(content),
                    }),
                    _ => Err(not_an_enum()),
                }
            }
            _ => Err(not_an_enum()),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit_struct seq tuple tuple_struct map struct
        identifier
    }
}

fn not_an_enum() -> Error {
    Error::type_error("an enum value must be a variant name or a map with a single key")
}

/// An enum variant and its content, if it has one.
struct JsonEnum {
    variant: String,
    content: Option<Value>,
}

impl<'de> EnumAccess<'de> for JsonEnum {
    type Error = Error;
    type Variant = JsonVariant;

    fn variant_seed<S>(self, seed: S) -> Result<(S::Value, JsonVariant), Error>
    where
        S: DeserializeSeed<'de>,
    {
        let variant = seed.deserialize(StringDeserializer::<Error>::new(self.variant))?;
        Ok((variant, JsonVariant(self.content)))
    }
}

/// The content of an enum variant.
struct JsonVariant(Option<Value>);

impl<'de> VariantAccess<'de> for JsonVariant {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        match self.0 {
            None | Some(Value::Null) => Ok(()),
            Some(_) => Err(Error::type_error("expected a unit variant")),
        }
    }

    fn newtype_variant_seed<S>(self, seed: S) -> Result<S::Value, Error>
    where
        S: DeserializeSeed<'de>,
    {
        match self.0 {
            Some(content) => seed.deserialize(JsonDe(content)),
            None => Err(Error::type_error("expected a newtype variant")),
        }
    }

    fn tuple_variant<V: Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value, Error> {
        match self.0 {
            Some(Value::Array(values)) => visitor.visit_seq(JsonSeq {
                iter: values.into_iter(),
            }),
            _ => Err(Error::type_error("expected a tuple variant")),
        }
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        match self.0 {
            Some(Value::Object(map)) => visitor.visit_map(JsonMap {
                iter: map.into_iter(),
                next_value: None,
            }),
            _ => Err(Error::type_error("expected a struct variant")),
        }
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
    use super::{from_reader, from_slice, from_str};
    use crate::error::ErrorKind;
    #[cfg(any(feature = "yaml", feature = "json", feature = "toml"))]
    use crate::testdata::values;
    use crate::testdata::{goldens, types};

    #[cfg(feature = "yaml")]
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

    #[cfg(feature = "yaml")]
    #[test]
    fn page_fenced_yml_tag() {
        let page: types::Page = from_str(goldens::PAGE_FENCED_YML).expect("yml");
        assert_eq!(page, values::page());
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn fields_only_fenced_yaml_golden() {
        let fields: types::FieldsOnly =
            from_str(goldens::FIELDS_ONLY_FENCED_YAML).expect("fields-only");
        assert_eq!(fields, values::fields_only());
    }

    #[cfg(feature = "yaml")]
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

    #[cfg(feature = "yaml")]
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

    #[cfg(feature = "yaml")]
    #[test]
    fn leading_prefix_is_ignored() {
        let page: types::Page = from_str(goldens::PAGE_LEADING_PREFIX).expect("prefix");
        assert_eq!(page, values::page());
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn body_only_two_sections_golden() {
        let body: types::BodyOnly = from_str(goldens::BODY_ONLY_TWO_SECTIONS).expect("body-only");
        assert_eq!(body, values::body_only());
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn fence_not_first_is_body_not_fields() {
        // The later fence is not read as fields, so the first front-matter
        // key in declaration order is missing.
        let err = from_str::<types::Page>(goldens::PAGE_FENCE_NOT_FIRST).expect_err("not fields");
        assert_eq!(err.kind(), ErrorKind::Type);
        assert!(err.to_string().contains("field1"), "{err}");
        let err = from_str::<types::FieldsOnly>(goldens::PAGE_FENCE_NOT_FIRST)
            .expect_err("fence is not fields");
        assert_eq!(err.kind(), ErrorKind::Body);
    }

    #[cfg(feature = "yaml")]
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

    #[cfg(feature = "yaml")]
    #[test]
    fn optional_middle_none_golden() {
        let got: types::OptionalBody =
            from_str(goldens::OPTIONAL_MIDDLE_NONE).expect("middle none");
        assert_eq!(got, values::optional_middle_none());
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn optional_trailing_none_golden() {
        let got: types::OptionalBody =
            from_str(goldens::OPTIONAL_TRAILING_NONE).expect("trailing none");
        assert_eq!(got, values::optional_trailing_none());
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn optional_leading_none_golden() {
        let got: types::OptionalBody =
            from_str(goldens::OPTIONAL_LEADING_NONE).expect("leading none");
        assert_eq!(got, values::optional_leading_none());
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn optional_empty_string_golden() {
        let got: types::OptionalBody =
            from_str(goldens::OPTIONAL_EMPTY_STRING).expect("empty string");
        assert_eq!(got, values::optional_empty_string());
    }

    #[cfg(feature = "yaml")]
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
        let value = from_str::<types::OptionalBody>(input).expect("the last field takes the rest");
        assert_eq!(value.first.as_deref(), Some("a"));
        assert_eq!(value.middle, None);
        assert_eq!(value.last.as_deref(), Some("c\n---\nextra\n"));
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn empty_section_for_required_structured_is_body() {
        let input = "```yaml\ntitle: t\n```\n---\n";
        let err = from_str::<types::Article>(input).expect_err("empty structured");
        assert_eq!(err.kind(), ErrorKind::Body);
        assert!(std::error::Error::source(&err).is_none());
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn structured_decoder_failure_is_body_with_source() {
        let input = "```yaml\ntitle: Hello\n```\n:\n";
        let err = from_str::<types::Article>(input).expect_err("bad structured yaml");
        assert_eq!(err.kind(), ErrorKind::Body);
        assert!(std::error::Error::source(&err).is_some());
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn nested_fenced_yaml_golden() {
        let got: types::NestedFields = from_str(goldens::NESTED_FENCED_YAML).expect("nested");
        assert_eq!(got, values::nested());
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn structured_fenced_yaml_golden() {
        let got: types::Article = from_str(goldens::STRUCTURED_FENCED_YAML).expect("structured");
        assert_eq!(got, values::article());
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn names_fenced_yaml_golden() {
        let got: types::JsonNames = from_str(goldens::NAMES_FENCED_YAML).expect("names");
        assert_eq!(got, values::json_names());
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn whitespace_unicode_golden_keeps_interiors() {
        let got: types::Page = from_str(goldens::WHITESPACE_UNICODE).expect("whitespace");
        assert_eq!(got, values::whitespace_page());
        assert!(got.text1.contains("spaces   \n"), "{:?}", got.text1);
        assert!(got.text1.contains("🦀"), "{:?}", got.text1);
        assert!(got.text1.contains("Ещё"), "{:?}", got.text1);
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn well_known_fenced_yaml_golden() {
        let got: types::WellKnown = from_str(goldens::WELL_KNOWN_FENCED_YAML).expect("well_known");
        assert_eq!(got, values::well_known());
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn well_known_generated_deserializes_golden() {
        let got: serde_markdown_proto::markdown::testdata::WellKnown =
            from_str(goldens::WELL_KNOWN_FENCED_YAML).expect("generated well_known");
        assert_eq!(got, values::well_known_generated());
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn page_published_yaml_golden() {
        let got: types::Page = from_str(goldens::PAGE_PUBLISHED_YAML).expect("published");
        assert_eq!(got, values::page_with_published());
        assert!(got.published.is_some());
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn page_published_generated_deserializes_golden() {
        let got: serde_markdown_proto::markdown::testdata::Page =
            from_str(goldens::PAGE_PUBLISHED_YAML).expect("generated published");
        assert_eq!(got, values::page_generated_with_published());
        assert!(got.published.is_set());
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn page_generated_deserializes_golden() {
        let got: serde_markdown_proto::markdown::testdata::Page =
            from_str(goldens::PAGE_FENCED_YAML).expect("generated page");
        assert_eq!(got, values::page_generated());
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn proto3_page_generated_deserializes_golden() {
        let got: serde_markdown_proto::markdown::testdata::Proto3Page =
            from_str(goldens::PROTO3_FENCED_YAML).expect("generated proto3");
        assert_eq!(got, values::proto3_page_generated());
        assert!(got.body.is_some());
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn json_names_generated_deserializes_golden() {
        let got: serde_markdown_proto::markdown::testdata::JsonNames =
            from_str(goldens::NAMES_FENCED_YAML).expect("generated names");
        assert_eq!(got, values::json_names_generated());
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn optional_body_generated_deserializes_golden() {
        let middle: serde_markdown_proto::markdown::testdata::OptionalBody =
            from_str(goldens::OPTIONAL_MIDDLE_NONE).expect("generated middle none");
        assert_eq!(middle, values::optional_middle_none_generated());
        let trailing: serde_markdown_proto::markdown::testdata::OptionalBody =
            from_str(goldens::OPTIONAL_TRAILING_NONE).expect("generated trailing none");
        assert_eq!(trailing, values::optional_trailing_none_generated());
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn article_generated_deserializes_golden() {
        let got: serde_markdown_proto::markdown::testdata::Article =
            from_str(goldens::STRUCTURED_FENCED_YAML).expect("generated article");
        assert_eq!(got, values::article_generated());
    }

    #[cfg(feature = "yaml")]
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
        let value = from_str::<types::Page>(input).expect("the last field takes the rest");
        assert_eq!(value.text1, "Text1 bla bla bla");
        assert_eq!(value.appendix, "Text2 bal bla bla\n---\nextra\n");
    }

    #[cfg(feature = "yaml")]
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

    #[cfg(feature = "yaml")]
    #[test]
    fn invalid_fields_yaml_is_front_matter() {
        let input = "```yaml\n:\n```\n";
        let err = from_str::<types::FieldsOnly>(input).expect_err("bad yaml");
        assert_eq!(err.kind(), ErrorKind::FrontMatter);
        assert!(std::error::Error::source(&err).is_some());
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn from_slice_matches_from_str() {
        let from_str: types::Page = from_str(goldens::PAGE_FENCED_YAML).expect("from_str");
        let from_slice: types::Page =
            from_slice(goldens::PAGE_FENCED_YAML.as_bytes()).expect("from_slice");
        assert_eq!(from_str, from_slice);
        assert_eq!(from_slice, values::page());
    }

    #[test]
    fn from_slice_invalid_utf8_is_syntax() {
        let err = from_slice::<types::FieldsOnly>(&[0xff, b'a']).expect_err("utf8");
        assert_eq!(err.kind(), ErrorKind::Syntax);
        assert_eq!(err.offset(), Some(0));
        assert!(std::error::Error::source(&err).is_none());
        assert!(err.to_string().contains("invalid UTF-8"), "{err}");
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn from_reader_matches_from_str() {
        let from_str: types::Page = from_str(goldens::PAGE_FENCED_YAML).expect("from_str");
        let from_reader: types::Page =
            from_reader(goldens::PAGE_FENCED_YAML.as_bytes()).expect("from_reader");
        assert_eq!(from_str, from_reader);
        assert_eq!(from_reader, values::page());
    }

    #[test]
    fn from_reader_io_error_is_io() {
        struct Boom;
        impl std::io::Read for Boom {
            fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("boom"))
            }
        }
        let err = from_reader::<_, types::FieldsOnly>(Boom).expect_err("read");
        assert_eq!(err.kind(), ErrorKind::Io);
        assert!(std::error::Error::source(&err).is_some());
    }

    #[test]
    fn from_reader_invalid_utf8_is_syntax() {
        let err = from_reader::<_, types::FieldsOnly>([0xff, b'a'].as_slice()).expect_err("utf8");
        assert_eq!(err.kind(), ErrorKind::Syntax);
        assert_eq!(err.offset(), Some(0));
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn unclosed_fence_from_str_is_syntax() {
        let err = from_str::<types::FieldsOnly>("```yaml\nname: only\n").expect_err("unclosed");
        assert_eq!(err.kind(), ErrorKind::Syntax);
        assert_eq!(err.offset(), Some(0));
        assert!(std::error::Error::source(&err).is_none());
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn empty_input_missing_required_body_is_body() {
        let err = from_str::<types::BodyOnly>("").expect_err("empty");
        assert_eq!(err.kind(), ErrorKind::Body);
        assert!(err.to_string().contains("text1"), "{err}");
        // With the front matter missing too, its first key is reported.
        let err = from_str::<types::Page>("").expect_err("empty");
        assert_eq!(err.kind(), ErrorKind::Type);
    }

    #[cfg(all(not(feature = "yaml"), feature = "json"))]
    #[test]
    fn without_yaml_prose_is_body_and_a_yaml_mapping_is_format_disabled() {
        let got: types::BodyOnly =
            from_str(goldens::BODY_ONLY_TWO_SECTIONS).expect("prose is the body");
        assert_eq!(got, values::body_only());
        let err = from_str::<types::FieldsOnly>("name: only\ncount: 2\n").expect_err("yaml");
        assert_eq!(err.kind(), ErrorKind::FormatDisabled);
    }
}
