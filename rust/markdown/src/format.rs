//! Fence language and fields-block layout used when serializing.
//!
//! [`Format`] and [`FieldsLayout`] live here. Dump and load of the fields
//! intermediate representation (a JSON value) go through the fence language.

use std::error::Error as StdError;
use std::fmt;

use serde::Serialize;
use serde_json::Value;

use crate::error::Error;

/// Fence language used when serializing the fields block.
///
/// Deserialize still honors an explicit fence tag on input; this enum is the
/// serialize choice. JSON and TOML are only written when the caller passes them.
/// A language not compiled into the crate (`yaml` / `json` / `toml` features)
/// is [`crate::ErrorKind::FormatDisabled`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Format {
    /// YAML (`yaml` / `yml` on input). Default serialize language, labeled `yaml`.
    #[default]
    Yaml,
    /// JSON, labeled `json` when fenced.
    Json,
    /// TOML, labeled `toml` when fenced.
    Toml,
}

/// Whether the serialized fields block is a fenced code block or a bare mapping.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FieldsLayout {
    /// Fenced code block (default serialize: ` ```yaml `).
    #[default]
    Fenced,
    /// Unfenced mapping; a `---` separator is written before the first body section.
    Bare,
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match *self {
            Self::Yaml => "yaml",
            Self::Json => "json",
            Self::Toml => "toml",
        })
    }
}

/// Encode `value` in `format`. The encoder writes keys in the order the
/// value's `Serialize` impl emits them.
pub(crate) fn dump<T: Serialize + ?Sized>(value: &T, format: Format) -> Result<String, Error> {
    match format {
        Format::Yaml => dump_yaml(value),
        Format::Json => dump_json(value),
        Format::Toml => dump_toml(value),
    }
}

/// Decode a fields slice in `format` to a JSON value.
pub(crate) fn load(src: &str, format: Format) -> Result<Value, Error> {
    decode(src, format, false)
}

/// Decode a structured body section in `format`.
///
/// Decoder failures are [`crate::ErrorKind::Body`] with `source` set.
/// [`crate::ErrorKind::FormatDisabled`] is unchanged.
pub(crate) fn load_body(src: &str, format: Format) -> Result<Value, Error> {
    decode(src, format, true)
}

fn decode(src: &str, format: Format, body: bool) -> Result<Value, Error> {
    match format {
        Format::Yaml => load_yaml(src, body),
        Format::Json => load_json(src, body),
        Format::Toml => load_toml(src, body),
    }
}

fn wrap_decoder<E>(err: E, body: bool) -> Error
where
    E: Into<Box<dyn StdError + Send + Sync + 'static>>,
{
    if body {
        Error::body_source("structured section", err)
    } else {
        Error::front_matter(err)
    }
}

fn dump_yaml<T: Serialize + ?Sized>(value: &T) -> Result<String, Error> {
    #[cfg(feature = "yaml")]
    {
        yaml_serde::to_string(value).map_err(Error::type_error)
    }
    #[cfg(not(feature = "yaml"))]
    {
        let _ = value;
        Err(Error::format_disabled(Format::Yaml))
    }
}

fn dump_json<T: Serialize + ?Sized>(value: &T) -> Result<String, Error> {
    #[cfg(feature = "json")]
    {
        serde_json::to_string_pretty(value).map_err(Error::type_error)
    }
    #[cfg(not(feature = "json"))]
    {
        let _ = value;
        Err(Error::format_disabled(Format::Json))
    }
}

fn dump_toml<T: Serialize + ?Sized>(value: &T) -> Result<String, Error> {
    #[cfg(feature = "toml")]
    {
        toml::to_string_pretty(value).map_err(Error::type_error)
    }
    #[cfg(not(feature = "toml"))]
    {
        let _ = value;
        Err(Error::format_disabled(Format::Toml))
    }
}

fn load_yaml(src: &str, body: bool) -> Result<Value, Error> {
    #[cfg(feature = "yaml")]
    {
        yaml_serde::from_str(src).map_err(|err| wrap_decoder(err, body))
    }
    #[cfg(not(feature = "yaml"))]
    {
        let _ = (src, body);
        Err(Error::format_disabled(Format::Yaml))
    }
}

fn load_json(src: &str, body: bool) -> Result<Value, Error> {
    #[cfg(feature = "json")]
    {
        serde_json::from_str(src).map_err(|err| wrap_decoder(err, body))
    }
    #[cfg(not(feature = "json"))]
    {
        let _ = (src, body);
        Err(Error::format_disabled(Format::Json))
    }
}

fn load_toml(src: &str, body: bool) -> Result<Value, Error> {
    #[cfg(feature = "toml")]
    {
        toml::from_str(src).map_err(|err| wrap_decoder(err, body))
    }
    #[cfg(not(feature = "toml"))]
    {
        let _ = (src, body);
        Err(Error::format_disabled(Format::Toml))
    }
}

#[cfg(test)]
mod tests {
    use super::{FieldsLayout, Format};

    #[test]
    fn defaults() {
        assert_eq!(Format::default(), Format::Yaml);
        assert_eq!(FieldsLayout::default(), FieldsLayout::Fenced);
    }

    #[test]
    fn format_display() {
        assert_eq!(Format::Yaml.to_string(), "yaml");
        assert_eq!(Format::Json.to_string(), "json");
        assert_eq!(Format::Toml.to_string(), "toml");
    }

    #[test]
    fn copy_eq() {
        let yaml = Format::Yaml;
        let yaml_copied = yaml;
        assert_eq!(yaml, yaml_copied);
        assert_ne!(Format::Yaml, Format::Json);

        let fenced = FieldsLayout::Fenced;
        let fenced_copied = fenced;
        assert_eq!(fenced, fenced_copied);
        assert_ne!(FieldsLayout::Fenced, FieldsLayout::Bare);
    }
}
