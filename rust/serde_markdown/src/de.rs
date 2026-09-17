//! Deserialize a `Markdown` root struct from a document.

use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

use crate::error::Error;
use crate::format::{self, Format};
use crate::markdown::Markdown;
use crate::parse::{self, Fields, Sniff};

/// Deserialize a Markdown document from `s`.
///
/// A first fenced fields block tagged `yaml` or `yml` is decoded as YAML.
/// Body fields listed in [`Markdown::BODY_FIELDS`] are filled from sections
/// in declaration order; `String` sections are the raw section text.
pub fn from_str<T>(s: &str) -> Result<T, Error>
where
    T: DeserializeOwned + Markdown,
{
    let doc = parse::parse(s);
    let mut fields = load_fields(&doc.fields)?;
    assign_body(&mut fields, T::BODY_FIELDS, &doc.body)?;
    serde_json::from_value(Value::Object(fields)).map_err(Error::type_error)
}

fn load_fields(fields: &Fields) -> Result<Map<String, Value>, Error> {
    let (format, inner) = match fields {
        Fields::Fenced {
            labeled,
            sniff,
            inner,
        } => (labeled.unwrap_or(*sniff), inner.as_str()),
        Fields::Bare { sniff, inner } => (*sniff, inner.as_str()),
    };
    let value = format::load(inner, sniff_format(format))?;
    match value {
        Value::Object(map) => Ok(map),
        Value::Null => Ok(Map::new()),
        _ => Err(Error::type_error("fields block must be a mapping")),
    }
}

fn sniff_format(sniff: Sniff) -> Format {
    match sniff {
        Sniff::Yaml => Format::Yaml,
        Sniff::Json => Format::Json,
        Sniff::Toml => Format::Toml,
    }
}

fn assign_body(
    fields: &mut Map<String, Value>,
    body_fields: &'static [&'static str],
    sections: &[String],
) -> Result<(), Error> {
    if sections.len() > body_fields.len() {
        return Err(Error::body(format!(
            "expected at most {} body section(s), found {}",
            body_fields.len(),
            sections.len()
        )));
    }
    if sections.len() < body_fields.len() {
        return Err(Error::body(format!(
            "expected {} body section(s), found {}",
            body_fields.len(),
            sections.len()
        )));
    }
    for (name, section) in body_fields.iter().zip(sections) {
        fields.insert(
            (*name).to_owned(),
            Value::String(section_text(section).to_owned()),
        );
    }
    Ok(())
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
