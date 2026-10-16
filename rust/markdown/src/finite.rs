//! A check that a value to be written holds no non-finite number.
//!
//! The fields IR is a JSON value, which cannot hold `NaN` or an infinity:
//! YAML writes `.inf`, TOML `inf`, and JSON writes `null`, which reads back as
//! an absent value. The writer refuses such a number before it encodes
//! anything, in every language.

use std::fmt;

use serde::ser::{
    self, Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTuple, SerializeTupleStruct, SerializeTupleVariant, Serializer,
};

/// Walks `value` and returns the text of the first non-finite number in it.
///
/// An error of the value's own `Serialize` impl is not reported here: the
/// encoder that follows meets it again and reports it with the right kind.
pub(crate) fn find_non_finite<T: Serialize + ?Sized>(value: &T) -> Option<String> {
    match value.serialize(Walk) {
        Err(WalkError::NonFinite(number)) => Some(number),
        Ok(()) | Err(WalkError::Other) => None,
    }
}

enum WalkError {
    NonFinite(String),
    Other,
}

impl fmt::Debug for WalkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("WalkError")
    }
}

impl fmt::Display for WalkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFinite(number) => write!(f, "the number {number} is not finite"),
            Self::Other => f.write_str("serialize error"),
        }
    }
}

impl std::error::Error for WalkError {}

impl ser::Error for WalkError {
    fn custom<T: fmt::Display>(_msg: T) -> Self {
        Self::Other
    }
}

/// A serializer that visits every part of a value and keeps nothing.
struct Walk;

macro_rules! walk_scalars {
    ($($method:ident($ty:ty)),* $(,)?) => {
        $(
            fn $method(self, _v: $ty) -> Result<(), WalkError> {
                Ok(())
            }
        )*
    };
}

impl Serializer for Walk {
    type Ok = ();
    type Error = WalkError;
    type SerializeSeq = Walk;
    type SerializeTuple = Walk;
    type SerializeTupleStruct = Walk;
    type SerializeTupleVariant = Walk;
    type SerializeMap = Walk;
    type SerializeStruct = Walk;
    type SerializeStructVariant = Walk;

    walk_scalars! {
        serialize_bool(bool),
        serialize_i8(i8),
        serialize_i16(i16),
        serialize_i32(i32),
        serialize_i64(i64),
        serialize_i128(i128),
        serialize_u8(u8),
        serialize_u16(u16),
        serialize_u32(u32),
        serialize_u64(u64),
        serialize_u128(u128),
        serialize_char(char),
        serialize_str(&str),
        serialize_bytes(&[u8]),
        serialize_unit_struct(&'static str),
    }

    fn serialize_f32(self, v: f32) -> Result<(), WalkError> {
        if v.is_finite() {
            Ok(())
        } else {
            Err(WalkError::NonFinite(v.to_string()))
        }
    }

    fn serialize_f64(self, v: f64) -> Result<(), WalkError> {
        if v.is_finite() {
            Ok(())
        } else {
            Err(WalkError::NonFinite(v.to_string()))
        }
    }

    fn serialize_none(self) -> Result<(), WalkError> {
        Ok(())
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<(), WalkError> {
        value.serialize(Walk)
    }

    fn serialize_unit(self) -> Result<(), WalkError> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<(), WalkError> {
        Ok(())
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<(), WalkError> {
        value.serialize(Walk)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<(), WalkError> {
        value.serialize(Walk)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Walk, WalkError> {
        Ok(Walk)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Walk, WalkError> {
        Ok(Walk)
    }

    fn serialize_tuple_struct(self, _name: &'static str, _len: usize) -> Result<Walk, WalkError> {
        Ok(Walk)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Walk, WalkError> {
        Ok(Walk)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Walk, WalkError> {
        Ok(Walk)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Walk, WalkError> {
        Ok(Walk)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Walk, WalkError> {
        Ok(Walk)
    }
}

impl SerializeSeq for Walk {
    type Ok = ();
    type Error = WalkError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), WalkError> {
        value.serialize(Walk)
    }

    fn end(self) -> Result<(), WalkError> {
        Ok(())
    }
}

impl SerializeTuple for Walk {
    type Ok = ();
    type Error = WalkError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), WalkError> {
        value.serialize(Walk)
    }

    fn end(self) -> Result<(), WalkError> {
        Ok(())
    }
}

impl SerializeTupleStruct for Walk {
    type Ok = ();
    type Error = WalkError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), WalkError> {
        value.serialize(Walk)
    }

    fn end(self) -> Result<(), WalkError> {
        Ok(())
    }
}

impl SerializeTupleVariant for Walk {
    type Ok = ();
    type Error = WalkError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), WalkError> {
        value.serialize(Walk)
    }

    fn end(self) -> Result<(), WalkError> {
        Ok(())
    }
}

impl SerializeMap for Walk {
    type Ok = ();
    type Error = WalkError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), WalkError> {
        key.serialize(Walk)
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), WalkError> {
        value.serialize(Walk)
    }

    fn end(self) -> Result<(), WalkError> {
        Ok(())
    }
}

impl SerializeStruct for Walk {
    type Ok = ();
    type Error = WalkError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), WalkError> {
        value.serialize(Walk)
    }

    fn end(self) -> Result<(), WalkError> {
        Ok(())
    }
}

impl SerializeStructVariant for Walk {
    type Ok = ();
    type Error = WalkError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), WalkError> {
        value.serialize(Walk)
    }

    fn end(self) -> Result<(), WalkError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use super::find_non_finite;

    #[derive(Serialize)]
    struct Nested {
        list: Vec<Option<f64>>,
        pair: (i32, f32),
    }

    #[test]
    fn finds_a_non_finite_number_anywhere_in_a_value() {
        assert_eq!(find_non_finite(&1.5_f64), None);
        assert_eq!(find_non_finite(&f64::NAN), Some("NaN".to_owned()));
        assert_eq!(find_non_finite(&f64::NEG_INFINITY), Some("-inf".to_owned()));
        let finite = Nested {
            list: vec![None, Some(2.0)],
            pair: (1, 0.5),
        };
        assert_eq!(find_non_finite(&finite), None);
        let list = Nested {
            list: vec![None, Some(f64::INFINITY)],
            pair: (1, 0.5),
        };
        assert_eq!(find_non_finite(&list), Some("inf".to_owned()));
        let pair = Nested {
            list: Vec::new(),
            pair: (1, f32::NAN),
        };
        assert_eq!(find_non_finite(&pair), Some("NaN".to_owned()));
    }
}
