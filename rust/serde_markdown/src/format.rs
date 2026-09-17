//! Fence language and fields-block layout used when serializing.
//!
//! [`Format`] and [`FieldsLayout`] live here. Dump and load of the fields
//! intermediate representation is not implemented in this file yet.

use std::fmt;

/// Fence language used when serializing the fields block.
///
/// Deserialize still honors an explicit fence tag on input; this enum is the
/// serialize choice. JSON and TOML are only written when the caller passes them.
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
