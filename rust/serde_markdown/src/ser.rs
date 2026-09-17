//! Serialize a `Markdown` root struct to a document.

use serde::ser::{Impossible, Serialize, SerializeStruct, Serializer};
use serde_json::{Map, Value};

use crate::error::Error;
use crate::format::{self, FieldsLayout, Format};
use crate::markdown::Markdown;

/// Serialize `value` as a fenced YAML Markdown document.
///
/// The fields block is a labeled `yaml` fence. Body `String` fields are written
/// as raw section text, joined with `\n---\n`. There is no leading blank line,
/// one newline after the closing fence, a trailing newline at EOF, and no
/// trailing `---` after the last section.
pub fn to_string<T: Serialize + Markdown>(value: &T) -> Result<String, Error> {
    to_string_with(value, Format::Yaml, FieldsLayout::Fenced)
}

fn to_string_with<T: Serialize + Markdown>(
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
    if layout != FieldsLayout::Fenced || format != Format::Yaml {
        return Err(Error::type_error(
            "only fenced YAML serialize is implemented",
        ));
    }

    let mut out = String::new();
    if !captured.fields.is_empty() {
        let yaml = format::dump(&Value::Object(captured.fields), Format::Yaml)?;
        out.push_str("```yaml\n");
        out.push_str(&yaml);
        if !yaml.ends_with('\n') {
            out.push('\n');
        }
        out.push_str("```\n");
    }

    for (i, name) in body_fields.iter().enumerate() {
        let Some(value) = captured.body.get(*name) else {
            return Err(Error::body(format!("missing body field {name}")));
        };
        let Value::String(section) = value else {
            return Err(Error::body(format!(
                "body field {name} must serialize as a string"
            )));
        };
        if i > 0 {
            if !out.ends_with('\n') {
                out.push('\n');
            }
            out.push_str("---\n");
        }
        out.push_str(section);
    }

    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use super::to_string;
    use crate::error::ErrorKind;
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
}
