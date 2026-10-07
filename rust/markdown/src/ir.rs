//! The fields IR: a decoded fields block or structured body section as a
//! `serde_json::Value`, the same for every fence language.
//!
//! [`Ir`] reads like `serde_json::Value`, with two differences that keep
//! what a language writes readable by the same mapping layer:
//!
//! - a YAML tagged value (`!Variant value`, how `yaml_serde` writes a
//!   newtype, tuple, or struct enum variant) becomes the single-key map
//!   `{Variant: value}`, the form JSON and TOML write;
//! - a TOML date-time, which the `toml` crate hands over as a map with one
//!   private key, becomes its text, so it reads into a string field or a
//!   well-known `Timestamp` like a quoted date does.

use std::fmt;

use serde::de::{
    self, Deserialize, Deserializer, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor,
};
use serde_json::{Map, Number, Value};

/// The key under which the `toml` crate hands over a TOML date-time.
const TOML_DATETIME: &str = "$__toml_private_datetime";

/// A value decoded into the fields IR.
pub(crate) struct Ir(pub(crate) Value);

impl<'de> Deserialize<'de> for Ir {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(IrVisitor).map(Ir)
    }
}

struct IrVisitor;

impl<'de> Visitor<'de> for IrVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a YAML, JSON, or TOML value")
    }

    fn visit_bool<E: de::Error>(self, b: bool) -> Result<Value, E> {
        Ok(Value::Bool(b))
    }

    fn visit_i64<E: de::Error>(self, n: i64) -> Result<Value, E> {
        Ok(Value::Number(n.into()))
    }

    fn visit_u64<E: de::Error>(self, n: u64) -> Result<Value, E> {
        Ok(Value::Number(n.into()))
    }

    fn visit_i128<E: de::Error>(self, n: i128) -> Result<Value, E> {
        if let Ok(n) = i64::try_from(n) {
            return self.visit_i64(n);
        }
        match u64::try_from(n) {
            Ok(n) => self.visit_u64(n),
            Err(_) => Err(E::custom(format!("integer {n} is out of the 64-bit range"))),
        }
    }

    fn visit_u128<E: de::Error>(self, n: u128) -> Result<Value, E> {
        match u64::try_from(n) {
            Ok(n) => self.visit_u64(n),
            Err(_) => Err(E::custom(format!("integer {n} is out of the 64-bit range"))),
        }
    }

    fn visit_f64<E: de::Error>(self, x: f64) -> Result<Value, E> {
        Ok(Number::from_f64(x).map_or(Value::Null, Value::Number))
    }

    fn visit_str<E: de::Error>(self, s: &str) -> Result<Value, E> {
        Ok(Value::String(s.to_owned()))
    }

    fn visit_string<E: de::Error>(self, s: String) -> Result<Value, E> {
        Ok(Value::String(s))
    }

    fn visit_bytes<E: de::Error>(self, bytes: &[u8]) -> Result<Value, E> {
        match std::str::from_utf8(bytes) {
            Ok(s) => self.visit_str(s),
            Err(_) => Err(E::custom("binary data is not UTF-8 text")),
        }
    }

    fn visit_none<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_unit<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_some<D: Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        Ir::deserialize(deserializer).map(|Ir(value)| value)
    }

    fn visit_newtype_struct<D: Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Value, D::Error> {
        Ir::deserialize(deserializer).map(|Ir(value)| value)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        let mut values = Vec::with_capacity(seq.size_hint().unwrap_or(0));
        while let Some(Ir(value)) = seq.next_element()? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        let mut fields = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            let Ir(value) = map.next_value()?;
            fields.insert(key, value);
        }
        if fields.len() == 1
            && let Some(Value::String(datetime)) = fields.get(TOML_DATETIME)
        {
            return Ok(Value::String(datetime.clone()));
        }
        Ok(Value::Object(fields))
    }

    fn visit_enum<A: EnumAccess<'de>>(self, data: A) -> Result<Value, A::Error> {
        let (variant, access) = data.variant::<String>()?;
        let Ir(value) = access.newtype_variant()?;
        let mut tagged = Map::new();
        tagged.insert(variant, value);
        Ok(Value::Object(tagged))
    }
}

#[cfg(test)]
mod tests {
    use super::Ir;

    #[cfg(feature = "yaml")]
    #[test]
    fn a_yaml_tagged_value_is_a_single_key_map() {
        let Ir(value) =
            yaml_serde::from_str("a: !Moved 3\nb: !Point {x: 1}\nc: Plain\n").expect("yaml");
        assert_eq!(
            value,
            serde_json::json!({"a": {"Moved": 3}, "b": {"Point": {"x": 1}}, "c": "Plain"})
        );
    }

    #[cfg(feature = "toml")]
    #[test]
    fn a_toml_date_time_is_its_text() {
        let Ir(value) = toml::from_str("when = 1979-05-27T07:32:00Z\n").expect("toml");
        assert_eq!(value, serde_json::json!({"when": "1979-05-27T07:32:00Z"}));
    }

    #[cfg(feature = "json")]
    #[test]
    fn json_reads_as_serde_json_does() {
        let text =
            r#"{"n": 1, "x": 1.5, "big": 18446744073709551615, "s": "t", "l": [null, true]}"#;
        let Ir(value) = serde_json::from_str(text).expect("json");
        let plain: serde_json::Value = serde_json::from_str(text).expect("json");
        assert_eq!(value, plain);
    }
}
