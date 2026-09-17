//! Deserialize a `Markdown` root struct from a document.

use serde::de::DeserializeOwned;
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
/// in declaration order; `String` sections are the raw section text.
///
/// Trailing omitted sections become `None` on optional fields. An empty
/// section is `None`; a section whose content is the two characters `""`
/// is `Some("")`. Extra sections, a missing required section, and an empty
/// section for a required structured field are [`ErrorKind::Body`].
pub fn from_str<T>(s: &str) -> Result<T, Error>
where
    T: DeserializeOwned + Markdown,
{
    let (mut fields, body) = decode_document(parse::parse(s))?;
    let used_absent = assign_body(&mut fields, T::BODY_FIELDS, &body)?;
    serde_json::from_value(Value::Object(fields)).map_err(|err| {
        if used_absent {
            Error::body(err)
        } else {
            Error::type_error(err)
        }
    })
}

fn decode_document(doc: Document) -> Result<(Map<String, Value>, Vec<String>), Error> {
    let Document { fields, body } = doc;
    match fields {
        Fields::Fenced {
            labeled,
            sniff,
            inner,
        } => {
            let format = sniff_format(labeled.unwrap_or(sniff));
            let map = mapping_or_type_error(format::load(&inner, format)?)?;
            Ok((map, body))
        }
        Fields::Bare { sniff, inner } => match load_bare_mapping(&inner, sniff)? {
            Some(map) => Ok((map, body)),
            None => {
                let mut sections = Vec::with_capacity(body.len() + 1);
                sections.push(inner);
                sections.extend(body);
                Ok((Map::new(), sections))
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

fn assign_body(
    fields: &mut Map<String, Value>,
    body_fields: &'static [&'static str],
    sections: &[String],
) -> Result<bool, Error> {
    if sections.len() > body_fields.len() {
        return Err(Error::body(format!(
            "expected at most {} body section(s), found {}",
            body_fields.len(),
            sections.len()
        )));
    }
    let mut used_absent = sections.len() < body_fields.len();
    for (i, name) in body_fields.iter().enumerate() {
        let value = match sections.get(i) {
            None => Value::Null,
            Some(section) => {
                let text = section_text(section);
                if text.is_empty() {
                    used_absent = true;
                    Value::Null
                } else if text == EMPTY_STRING_SECTION {
                    Value::String(String::new())
                } else {
                    Value::String(text.to_owned())
                }
            }
        };
        fields.insert((*name).to_owned(), value);
    }
    Ok(used_absent)
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
