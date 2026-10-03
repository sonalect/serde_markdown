//! CommonMark split of a Markdown document into a fields slice and body sections.
//!
//! This module decodes YAML, JSON, or TOML only for a bare first slice, to
//! decide whether that slice is a mapping (fields) or the start of the body.
//! A fenced fields block is returned as text. An unclosed fenced code block
//! before the last body section is [`crate::ErrorKind::Syntax`] with a byte
//! offset.
//!
//! The last body field is never parsed: it is every byte after its separator.
//! Only the text before it is scanned for top-level dash `---` rules.

use std::ops::Range;

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use serde_json::{Map, Value};

use crate::error::{Error, ErrorKind};
use crate::format::{self, Format};

/// Shape-based guess for an unlabeled fence or a bare first slice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Sniff {
    Yaml,
    Json,
    Toml,
}

impl Sniff {
    /// The fence language this guess selects.
    pub(crate) fn format(self) -> Format {
        match self {
            Self::Yaml => Format::Yaml,
            Self::Json => Format::Json,
            Self::Toml => Format::Toml,
        }
    }
}

/// How the first fields slice was found.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Fields {
    /// First top-level block after the prefix is a yaml/json/toml (or unlabeled) fence.
    Fenced {
        /// `None` when the info string was empty (sniff applies).
        labeled: Option<Sniff>,
        sniff: Sniff,
        inner: String,
    },
    /// No fields fence; text until the next top-level `---` (or EOF) that
    /// decoded as a mapping.
    Bare {
        sniff: Sniff,
        inner: String,
        map: Map<String, Value>,
    },
    /// No fields. The body starts at the first byte of the document.
    None,
}

/// Split result. Body sections do not include the `---` separators.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Document {
    pub fields: Fields,
    pub body: Vec<String>,
}

/// Split `input` for a root struct with `body_fields` body fields.
///
/// The first `body_fields - 1` top-level dash rules after the fields block
/// end the earlier sections; the last section is the rest of the document,
/// verbatim. Sections that are missing from the document are not returned.
pub(crate) fn parse(input: &str, body_fields: usize) -> Result<Document, Error> {
    let events: Vec<(Event<'_>, Range<usize>)> = Parser::new_ext(input, parser_options())
        .into_offset_iter()
        .collect();

    let dash_rules = top_level_dash_rules(input, &events);
    let content_start = skip_prefix(input, &dash_rules);

    let (fields, body_start, separated) = match leading_fields_fence(input, &events, content_start)
    {
        Some(fence) => {
            let fields = Fields::Fenced {
                labeled: fence.labeled,
                sniff: fence.sniff,
                inner: fence.inner,
            };
            (fields, fence.body_start, false)
        }
        None => {
            let next_rule = dash_rules
                .iter()
                .find(|range| range.start >= content_start)
                .cloned();
            let (inner, body_start, separated) = match next_rule {
                Some(range) => (
                    input[content_start..range.start].to_owned(),
                    range.end,
                    true,
                ),
                None => (input[content_start..].to_owned(), input.len(), false),
            };
            let sniff = sniff(&inner);
            match load_bare_mapping(&inner, sniff)? {
                Some(map) => (Fields::Bare { sniff, inner, map }, body_start, separated),
                None => (Fields::None, 0, false),
            }
        }
    };

    let split = split_body(input, body_start, &dash_rules, body_fields, separated);
    if let Some(offset) = unclosed_fence_offset(input, &events, split.verbatim_from) {
        return Err(Error::syntax(offset, "unclosed fence"));
    }
    Ok(Document {
        fields,
        body: split.sections,
    })
}

/// Bare first slice: a mapping is fields. YAML that is not a mapping (or that
/// the YAML decoder rejects) is `None`: the slice is body. A JSON or TOML
/// decode failure stays [`ErrorKind::FrontMatter`].
fn load_bare_mapping(inner: &str, sniff: Sniff) -> Result<Option<Map<String, Value>>, Error> {
    let format = sniff.format();
    let value = match format::load(inner, format) {
        Ok(value) => value,
        Err(err) if err.kind() == ErrorKind::FormatDisabled => return Err(err),
        Err(err) if matches!(format, Format::Json | Format::Toml) => return Err(err),
        Err(_) => return Ok(None),
    };
    Ok(match value {
        Value::Object(map) => Some(map),
        Value::Null => Some(Map::new()),
        _ => None,
    })
}

/// Whether a reader would take the start of `candidate` for a fields block
/// (or fail while trying to read it as one). Used by the writer to decide
/// whether a body-only document needs an explicit empty fields block.
pub(crate) fn would_read_fields(candidate: &str, body_fields: usize) -> Result<bool, Error> {
    match parse(candidate, body_fields) {
        Ok(doc) => Ok(!matches!(doc.fields, Fields::None)),
        Err(err)
            if matches!(
                err.kind(),
                ErrorKind::FrontMatter | ErrorKind::FormatDisabled
            ) =>
        {
            Ok(true)
        }
        Err(err) => Err(err),
    }
}

/// Canonical separator between two body sections: the line break that ends
/// the earlier section, then `---`, then its line break.
pub(crate) const SEPARATOR: &str = "\n---\n";

/// Check that `text` can be written as a body section that is not the last
/// one: followed by the canonical separator, a reader finds exactly that one
/// top-level dash rule and no unclosed fence.
pub(crate) fn check_middle_section(text: &str) -> Result<(), Error> {
    if text.ends_with('\r') {
        return Err(Error::body(
            "a body value that is not the last must not end with a carriage return",
        ));
    }
    let mut doc = String::with_capacity(text.len() + SEPARATOR.len());
    doc.push_str(text);
    doc.push_str(SEPARATOR);
    let events: Vec<(Event<'_>, Range<usize>)> = Parser::new_ext(&doc, parser_options())
        .into_offset_iter()
        .collect();
    let rules = top_level_dash_rules(&doc, &events);
    let closed = unclosed_fence_offset(&doc, &events, doc.len()).is_none();
    let expected = text.len() + 1;
    match rules.as_slice() {
        [rule] if closed && rule.start == expected => Ok(()),
        _ => Err(Error::body(
            "a body value that is not the last contains a top-level `---` line \
             (or an unclosed code fence) and would split on read; \
             put it in the last body field",
        )),
    }
}

/// Whether the first line of `text` is a dash thematic break.
pub(crate) fn starts_with_dash_rule(text: &str) -> bool {
    first_line(text).is_some_and(|(line, _)| is_dash_thematic_break(line))
}

/// First line of `text` without its terminator, and the text after the
/// terminator. `None` for empty text.
fn first_line(text: &str) -> Option<(&str, &str)> {
    if text.is_empty() {
        return None;
    }
    match text.find('\n') {
        Some(i) => Some((text[..i].trim_end_matches('\r'), &text[i + 1..])),
        None => Some((text.trim_end_matches('\r'), "")),
    }
}

/// Start of the first fenced block that begins before `limit` and has no
/// closing fence. Text from `limit` on is the verbatim last section and is
/// not checked.
fn unclosed_fence_offset(
    input: &str,
    events: &[(Event<'_>, Range<usize>)],
    limit: usize,
) -> Option<usize> {
    for (i, (event, range)) in events.iter().enumerate() {
        let Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(_))) = event else {
            continue;
        };
        if range.start >= limit {
            break;
        }
        let end_idx = matching_code_block_end(events, i);
        let end = events
            .get(end_idx)
            .map(|(_, r)| r.end)
            .unwrap_or(input.len())
            .max(range.end)
            .min(input.len());
        let line_start = input[..range.start].rfind('\n').map_or(0, |nl| nl + 1);
        if !fenced_block_has_closer(&input[line_start..end], range.start - line_start) {
            return Some(range.start);
        }
    }
    None
}

/// Opening fence plus a last line that is a CommonMark closing fence of the
/// same character and at least the same length.
///
/// `src` starts at the beginning of the opener's raw line; `block_col` is
/// where the block starts on that line. Inside a list item or a block quote
/// the raw line carries the container's indentation or `>` markers, so the
/// closer is judged by its column against the opener's column (at most three
/// more), after a prefix of spaces and `>` markers, not by a fixed three
/// spaces from the line start. At top level this is the CommonMark rule for
/// an opener at column 0; for an indented opener it accepts a few columns
/// more than CommonMark would.
fn fenced_block_has_closer(src: &str, block_col: usize) -> bool {
    let trimmed = src.trim_end_matches(['\n', '\r']);
    let Some(first_nl) = trimmed.find('\n') else {
        return false;
    };
    let opener_line = trimmed[..first_nl].trim_end_matches('\r');
    let Some(from_block) = opener_line.get(block_col..) else {
        return false;
    };
    let open_col = block_col + (from_block.len() - from_block.trim_start_matches(' ').len());
    let Some((ch, n)) = opener_line.get(open_col..).and_then(fence_opener) else {
        return false;
    };
    let rest = trimmed[first_nl + 1..].trim_start_matches('\r');
    let last = match rest.rsplit_once('\n') {
        Some((_, last)) => last.trim_end_matches('\r'),
        None => rest.trim_end_matches('\r'),
    };
    let fence = last.trim_start_matches([' ', '>']);
    let close_col = last.len() - fence.len();
    close_col <= open_col + 3 && is_fence_closer(fence, ch, n)
}

fn fence_opener(line: &str) -> Option<(u8, usize)> {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() && i < 3 && bytes[i] == b' ' {
        i += 1;
    }
    let ch = *bytes.get(i)?;
    if ch != b'`' && ch != b'~' {
        return None;
    }
    let start = i;
    while i < bytes.len() && bytes[i] == ch {
        i += 1;
    }
    let n = i - start;
    if n < 3 {
        return None;
    }
    Some((ch, n))
}

fn is_fence_closer(line: &str, ch: u8, min: usize) -> bool {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() && i < 3 && bytes[i] == b' ' {
        i += 1;
    }
    let start = i;
    while i < bytes.len() && bytes[i] == ch {
        i += 1;
    }
    let n = i - start;
    if n < min {
        return false;
    }
    bytes[i..].iter().all(|b| *b == b' ' || *b == b'\t')
}

fn parser_options() -> Options {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_DEFINITION_LIST);
    options
}

fn nest_delta(event: &Event<'_>) -> i32 {
    match event {
        Event::Start(
            Tag::BlockQuote(_)
            | Tag::List(_)
            | Tag::Item
            | Tag::Table(_)
            | Tag::FootnoteDefinition(_)
            | Tag::DefinitionList
            | Tag::HtmlBlock
            | Tag::CodeBlock(_),
        ) => 1,
        Event::End(
            TagEnd::BlockQuote(_)
            | TagEnd::List(_)
            | TagEnd::Item
            | TagEnd::Table
            | TagEnd::FootnoteDefinition
            | TagEnd::DefinitionList
            | TagEnd::HtmlBlock
            | TagEnd::CodeBlock,
        ) => -1,
        _ => 0,
    }
}

fn top_level_dash_rules(input: &str, events: &[(Event<'_>, Range<usize>)]) -> Vec<Range<usize>> {
    let mut nest = 0i32;
    let mut rules = Vec::new();
    for (event, range) in events {
        nest += nest_delta(event);
        if nest != 0 {
            continue;
        }
        if matches!(event, Event::Rule) && is_dash_thematic_break(&input[range.clone()]) {
            rules.push(range.clone());
            continue;
        }
        if let Event::Start(Tag::Heading { .. }) = event
            && let Some(underline) = setext_dash_underline(input, range.clone())
        {
            rules.push(underline);
        }
    }
    rules
}

/// CommonMark turns `paragraph\n---` into a setext H2, not `Event::Rule`.
/// That underline is still a top-level dash separator.
fn setext_dash_underline(input: &str, heading: Range<usize>) -> Option<Range<usize>> {
    let src = &input[heading.clone()];
    let first_line_end = src.find('\n').unwrap_or(src.len());
    if src[..first_line_end].trim_start().starts_with('#') {
        return None;
    }
    let without_final_nl = src.trim_end_matches(['\n', '\r']);
    let last_nl = without_final_nl.rfind('\n')?;
    let last_line = &without_final_nl[last_nl + 1..];
    if !is_dash_thematic_break(last_line) {
        return None;
    }
    Some(heading.start + last_nl + 1..heading.end)
}

/// CommonMark dash thematic break: 0–3 spaces, three or more `-`, optional spaces between markers.
fn is_dash_thematic_break(src: &str) -> bool {
    let line = src.trim_end_matches(['\r', '\n']);
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() && i < 3 && bytes[i] == b' ' {
        i += 1;
    }
    let rest = &line[i..];
    let mut dashes = 0usize;
    for c in rest.chars() {
        match c {
            '-' => dashes += 1,
            ' ' | '\t' => {}
            _ => return false,
        }
    }
    dashes >= 3
}

fn skip_one_line_ending(input: &str, pos: usize) -> usize {
    match input.as_bytes().get(pos..) {
        Some([b'\r', b'\n', ..]) => pos + 2,
        Some([b'\n', ..] | [b'\r', ..]) => pos + 1,
        _ => pos,
    }
}

fn skip_ws(input: &str, mut pos: usize) -> usize {
    while pos < input.len() {
        let ch = input[pos..].chars().next();
        let Some(ch) = ch else {
            break;
        };
        if !ch.is_whitespace() {
            break;
        }
        pos += ch.len_utf8();
    }
    pos
}

fn skip_prefix(input: &str, dash_rules: &[Range<usize>]) -> usize {
    let mut pos = 0;
    loop {
        pos = skip_ws(input, pos);
        let Some(rule) = dash_rules.iter().find(|range| {
            range.start == pos
                || (range.start <= pos && pos < range.end)
                || skip_ws(input, range.start) == pos
        }) else {
            return pos;
        };
        pos = rule.end;
    }
}

struct LeadingFence {
    labeled: Option<Sniff>,
    sniff: Sniff,
    inner: String,
    body_start: usize,
}

fn leading_fields_fence(
    input: &str,
    events: &[(Event<'_>, Range<usize>)],
    content_start: usize,
) -> Option<LeadingFence> {
    let mut nest = 0i32;
    for (i, (event, range)) in events.iter().enumerate() {
        if range.end <= content_start {
            nest += nest_delta(event);
            continue;
        }
        if nest == 0
            && let Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) = event
        {
            let between = &input[content_start..range.start];
            if !between.chars().all(char::is_whitespace) {
                return None;
            }
            let labeled = fields_lang(info)?;
            let end_idx = matching_code_block_end(events, i);
            let inner = fenced_inner(input, events, i, end_idx);
            let sniff = labeled.unwrap_or_else(|| sniff(&inner));
            return Some(LeadingFence {
                labeled,
                sniff,
                inner,
                // The closing fence line's terminator is not body text.
                body_start: skip_one_line_ending(input, range.end),
            });
        }
        nest += nest_delta(event);
    }
    None
}

fn matching_code_block_end(events: &[(Event<'_>, Range<usize>)], start_idx: usize) -> usize {
    let mut depth = 0i32;
    for (j, (event, _)) in events[start_idx..].iter().enumerate() {
        match event {
            Event::Start(Tag::CodeBlock(_)) => depth += 1,
            Event::End(TagEnd::CodeBlock) => {
                depth -= 1;
                if depth == 0 {
                    return start_idx + j;
                }
            }
            _ => {}
        }
    }
    events.len().saturating_sub(1)
}

fn fenced_inner(
    input: &str,
    events: &[(Event<'_>, Range<usize>)],
    start_idx: usize,
    end_idx: usize,
) -> String {
    let mut inner = String::new();
    for (event, range) in &events[start_idx + 1..end_idx] {
        if matches!(event, Event::Text(_)) {
            inner.push_str(&input[range.clone()]);
        }
    }
    inner
}

/// `None` = this fence is not a fields fence. `Some(None)` = unlabeled. `Some(Some(lang))` = tagged.
fn fields_lang(info: &str) -> Option<Option<Sniff>> {
    let first = info.split_whitespace().next().unwrap_or("");
    if first.is_empty() {
        return Some(None);
    }
    match first.to_ascii_lowercase().as_str() {
        "yaml" | "yml" => Some(Some(Sniff::Yaml)),
        "json" => Some(Some(Sniff::Json)),
        "toml" => Some(Some(Sniff::Toml)),
        _ => None,
    }
}

fn sniff(text: &str) -> Sniff {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Sniff::Yaml;
    }
    let first = trimmed.chars().find(|c| !c.is_whitespace());
    match first {
        Some('{') | Some('[') => Sniff::Json,
        _ if looks_like_toml(trimmed) => Sniff::Toml,
        _ => Sniff::Yaml,
    }
}

fn looks_like_toml(text: &str) -> bool {
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with("[[") {
            return true;
        }
        if line.starts_with('[') && line.contains(']') {
            return true;
        }
        if let Some((key, _)) = line.split_once('=') {
            return is_toml_key(key.trim());
        }
        return false;
    }
    false
}

fn is_toml_key(key: &str) -> bool {
    if key.is_empty() {
        return false;
    }
    let quoted = (key.starts_with('"') && key.ends_with('"'))
        || (key.starts_with('\'') && key.ends_with('\''));
    if quoted {
        return key.len() >= 2;
    }
    key.chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
}

/// Sections of the body region plus where the verbatim last section starts.
struct Split {
    sections: Vec<String>,
    /// Byte offset where the last section's verbatim text begins, or the end
    /// of the document when the document has fewer separators than that.
    verbatim_from: usize,
}

/// Split the body region that starts at `from`.
///
/// `separated` is true when a `---` line closed the bare fields slice just
/// before `from`: that line is already the first separator, so a single body
/// field takes the region as is, even when it is empty.
fn split_body(
    input: &str,
    from: usize,
    dash_rules: &[Range<usize>],
    body_fields: usize,
    separated: bool,
) -> Split {
    let rest = input.get(from..).unwrap_or("");
    match body_fields {
        0 => {
            let sections = if rest.chars().all(char::is_whitespace) {
                Vec::new()
            } else {
                vec![rest.to_owned()]
            };
            Split {
                sections,
                verbatim_from: from,
            }
        }
        1 => {
            // The only body field is the last one: the region, verbatim. With
            // no `---` line before it (a fenced block, or no fields), an
            // empty value, or one whose first line is itself a dash rule,
            // is introduced by one `---` line that this field's reader drops.
            let sections = if separated {
                vec![rest.to_owned()]
            } else if let Some((line, after)) = first_line(rest)
                && is_dash_thematic_break(line)
            {
                vec![after.to_owned()]
            } else if rest.is_empty() {
                Vec::new()
            } else {
                vec![rest.to_owned()]
            };
            Split {
                sections,
                verbatim_from: from,
            }
        }
        _ => split_many(input, from, dash_rules, body_fields),
    }
}

fn split_many(input: &str, from: usize, dash_rules: &[Range<usize>], body_fields: usize) -> Split {
    if from >= input.len() {
        return Split {
            sections: Vec::new(),
            verbatim_from: input.len(),
        };
    }
    let mut sections = Vec::with_capacity(body_fields.min(8));
    let mut cur = from;
    let mut taken = 0;
    for range in dash_rules.iter().filter(|range| range.start >= from) {
        if taken == body_fields - 1 {
            break;
        }
        let line_start = input[..range.start].rfind('\n').map_or(0, |nl| nl + 1);
        let end = line_start.max(cur);
        sections.push(drop_line_end(&input[cur..end]).to_owned());
        cur = range.end;
        taken += 1;
    }
    let last = &input[cur..];
    // The last field takes the rest, even when empty, once all its
    // separators were seen. Short of that, the tail is an ordinary section
    // that ended at the end of the document, and nothing is a section.
    if taken == body_fields - 1 {
        sections.push(last.to_owned());
        Split {
            sections,
            verbatim_from: cur,
        }
    } else {
        if !last.is_empty() {
            sections.push(last.to_owned());
        }
        Split {
            sections,
            verbatim_from: input.len(),
        }
    }
}

/// Drop the one line terminator that belongs to the separator after a
/// section. Every other byte, trailing spaces and blank lines included, stays.
fn drop_line_end(section: &str) -> &str {
    section
        .strip_suffix("\r\n")
        .or_else(|| section.strip_suffix('\n'))
        .unwrap_or(section)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testdata::goldens;

    fn fenced(doc: &Document) -> (&Option<Sniff>, Sniff, &str) {
        match &doc.fields {
            Fields::Fenced {
                labeled,
                sniff,
                inner,
            } => (labeled, *sniff, inner.as_str()),
            Fields::Bare { .. } | Fields::None => panic!("expected fenced fields, got {doc:?}"),
        }
    }

    fn bare(doc: &Document) -> (Sniff, &str) {
        match &doc.fields {
            Fields::Bare { sniff, inner, .. } => (*sniff, inner.as_str()),
            Fields::Fenced { .. } | Fields::None => panic!("expected bare fields, got {doc:?}"),
        }
    }

    /// Split for a root with two body fields, like the `Page` fixture.
    fn parse_ok(input: &str) -> Document {
        parse_n(input, 2)
    }

    fn parse_n(input: &str, body_fields: usize) -> Document {
        parse(input, body_fields).unwrap_or_else(|err| panic!("{err}"))
    }

    #[test]
    fn sniff_empty_is_yaml() {
        assert_eq!(sniff(""), Sniff::Yaml);
        assert_eq!(sniff("  \n"), Sniff::Yaml);
    }

    #[test]
    fn sniff_json_object_and_array() {
        assert_eq!(sniff("{\n  \"a\": 1\n}"), Sniff::Json);
        assert_eq!(sniff("[1, 2]"), Sniff::Json);
    }

    #[test]
    fn sniff_toml_key_equals() {
        assert_eq!(sniff("field1 = \"foo\"\nfield3 = 1\n"), Sniff::Toml);
        assert_eq!(sniff("# comment\nfield1 = 1\n"), Sniff::Toml);
    }

    #[test]
    fn sniff_yaml_mapping() {
        assert_eq!(sniff("field1: foo\nfield3: 1\n"), Sniff::Yaml);
    }

    #[test]
    fn page_fenced_yaml() {
        let doc = parse_ok(goldens::PAGE_FENCED_YAML);
        let (labeled, sniff, inner) = fenced(&doc);
        assert_eq!(*labeled, Some(Sniff::Yaml));
        assert_eq!(sniff, Sniff::Yaml);
        assert!(inner.contains("field1: foo"));
        assert_eq!(doc.body.len(), 2);
        assert!(
            !doc.body[0].starts_with('\n'),
            "newline after the closing fence is not body: {:?}",
            doc.body[0]
        );
        assert_eq!(doc.body[0].trim(), "Text1 bla bla bla");
        assert_eq!(doc.body[1].trim(), "Text2 bal bla bla");
    }

    #[test]
    fn page_fenced_yml_tag() {
        let doc = parse_ok(goldens::PAGE_FENCED_YML);
        let (labeled, sniff, _) = fenced(&doc);
        assert_eq!(*labeled, Some(Sniff::Yaml));
        assert_eq!(sniff, Sniff::Yaml);
        assert_eq!(doc.body.len(), 2);
    }

    #[test]
    fn page_fenced_json_and_toml_tags() {
        let json = parse_ok(goldens::PAGE_FENCED_JSON);
        let (labeled, sniff, inner) = fenced(&json);
        assert_eq!(*labeled, Some(Sniff::Json));
        assert_eq!(sniff, Sniff::Json);
        assert!(inner.contains("\"field1\""));
        assert_eq!(json.body.len(), 2);

        let toml = parse_ok(goldens::PAGE_FENCED_TOML);
        let (labeled, sniff, inner) = fenced(&toml);
        assert_eq!(*labeled, Some(Sniff::Toml));
        assert_eq!(sniff, Sniff::Toml);
        assert!(inner.contains("field1 ="));
        assert_eq!(toml.body.len(), 2);
    }

    #[test]
    fn unlabeled_fence_sniffs() {
        let yaml = parse_ok(goldens::PAGE_UNLABELED_YAML);
        let (labeled, sniff, _) = fenced(&yaml);
        assert_eq!(*labeled, None);
        assert_eq!(sniff, Sniff::Yaml);
        assert_eq!(yaml.body.len(), 2);

        let json = parse_ok(goldens::PAGE_UNLABELED_JSON);
        let (labeled, sniff, _) = fenced(&json);
        assert_eq!(*labeled, None);
        assert_eq!(sniff, Sniff::Json);

        let toml = parse_ok(goldens::PAGE_UNLABELED_TOML);
        let (labeled, sniff, _) = fenced(&toml);
        assert_eq!(*labeled, None);
        assert_eq!(sniff, Sniff::Toml);
    }

    #[test]
    fn leading_prefix_dash_is_discarded() {
        let doc = parse_ok(goldens::PAGE_LEADING_PREFIX);
        let (labeled, _, _) = fenced(&doc);
        assert_eq!(*labeled, Some(Sniff::Yaml));
        assert_eq!(doc.body.len(), 2);
        assert_eq!(doc.body[0].trim(), "Text1 bla bla bla");
        assert_eq!(doc.body[1].trim(), "Text2 bal bla bla");
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn fence_not_first_is_body() {
        let doc = parse_ok(goldens::PAGE_FENCE_NOT_FIRST);
        assert_eq!(doc.fields, Fields::None);
        assert_eq!(doc.body.len(), 1);
        assert!(doc.body[0].starts_with("Intro paragraph"));
        assert!(doc.body[0].contains("```yaml"));
    }

    #[test]
    fn bare_slices_sniff() {
        #[cfg(feature = "yaml")]
        {
            let yaml = parse_ok(goldens::PAGE_BARE_YAML);
            let (sniff, inner) = bare(&yaml);
            assert_eq!(sniff, Sniff::Yaml);
            assert!(inner.contains("field1: foo"));
            assert_eq!(yaml.body.len(), 2);
        }
        #[cfg(feature = "json")]
        {
            let json = parse_ok(goldens::PAGE_BARE_JSON);
            let (sniff, _) = bare(&json);
            assert_eq!(sniff, Sniff::Json);
            assert_eq!(json.body.len(), 2);
        }
        #[cfg(feature = "toml")]
        {
            let toml = parse_ok(goldens::PAGE_BARE_TOML);
            let (sniff, _) = bare(&toml);
            assert_eq!(sniff, Sniff::Toml);
            assert_eq!(toml.body.len(), 2);
        }
    }

    #[test]
    fn dash_inside_body_fence_does_not_split() {
        let doc = parse_ok(goldens::PAGE_SPLIT_FENCE);
        assert_eq!(doc.body.len(), 2);
        assert!(doc.body[0].contains("---"));
        assert!(doc.body[0].contains("not a split"));
        assert_eq!(doc.body[1].trim(), "Text2 bal bla bla");
    }

    #[test]
    fn stars_and_underscores_do_not_split() {
        let doc = parse_ok(goldens::PAGE_SPLIT_STARS);
        assert_eq!(doc.body.len(), 2);
        assert!(doc.body[0].contains("***"));
        assert!(doc.body[0].contains("___"));
        assert!(doc.body[0].contains("still text1"));
        assert_eq!(doc.body[1].trim(), "Text2 bal bla bla");
    }

    #[test]
    fn dash_in_list_item_does_not_split() {
        let doc = parse_ok(goldens::PAGE_SPLIT_LIST);
        assert_eq!(doc.body.len(), 2);
        assert!(doc.body[0].contains("keep going"));
        assert!(doc.body[0].contains("---"));
        assert_eq!(doc.body[1].trim(), "Text2 bal bla bla");
    }

    #[test]
    fn fields_only_fenced_has_no_body() {
        let doc = parse_ok(goldens::FIELDS_ONLY_FENCED_YAML);
        let (labeled, _, inner) = fenced(&doc);
        assert_eq!(*labeled, Some(Sniff::Yaml));
        assert!(inner.contains("name: only"));
        assert!(doc.body.is_empty());
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn fields_only_bare_is_whole_file() {
        let doc = parse_ok(goldens::FIELDS_ONLY_BARE_YAML);
        let (sniff, inner) = bare(&doc);
        assert_eq!(sniff, Sniff::Yaml);
        assert!(inner.contains("name: only"));
        assert!(doc.body.is_empty());
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn body_only_is_two_sections() {
        let doc = parse_ok(goldens::BODY_ONLY_TWO_SECTIONS);
        assert_eq!(doc.fields, Fields::None);
        assert_eq!(doc.body, ["Text1 bla bla bla", "Text2 bal bla bla"]);
    }

    #[test]
    fn optional_middle_none_keeps_empty_slot() {
        let doc = parse_n(goldens::OPTIONAL_MIDDLE_NONE, 3);
        assert_eq!(doc.body.len(), 3);
        assert_eq!(doc.body[0].trim(), "a");
        assert!(doc.body[1].trim().is_empty());
        assert_eq!(doc.body[2].trim(), "c");
    }

    #[test]
    fn optional_leading_none_empty_first_section() {
        let doc = parse_n(goldens::OPTIONAL_LEADING_NONE, 3);
        assert_eq!(doc.body.len(), 2);
        assert!(doc.body[0].trim().is_empty());
        assert_eq!(doc.body[1].trim(), "a");
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn rust_info_string_is_not_fields_fence() {
        let input = "```rust\nlet x = 1;\n```\n";
        let doc = parse_ok(input);
        assert_eq!(doc.fields, Fields::None);
        assert_eq!(doc.body, [input]);
    }

    #[test]
    fn tilde_yaml_fence_is_fields() {
        let input = "~~~yaml\nfield1: foo\n~~~\nbody\n";
        let doc = parse_ok(input);
        let (labeled, _, inner) = fenced(&doc);
        assert_eq!(*labeled, Some(Sniff::Yaml));
        assert!(inner.contains("field1: foo"));
        assert_eq!(doc.body.len(), 1);
        assert_eq!(doc.body[0].trim(), "body");
    }

    #[test]
    fn atx_heading_is_not_a_split() {
        let input = "```yaml\nk: 1\n```\n## still section one\ntext\n---\nsection two\n";
        let doc = parse_ok(input);
        assert_eq!(doc.body.len(), 2);
        assert!(doc.body[0].contains("## still section one"));
        assert_eq!(doc.body[1].trim(), "section two");
    }

    #[test]
    fn blank_line_dash_is_rule_split() {
        let input = "```yaml\nk: 1\n```\npara\n\n---\n\nnext\n";
        let doc = parse_ok(input);
        assert_eq!(doc.body.len(), 2);
        assert_eq!(doc.body[0].trim(), "para");
        assert_eq!(doc.body[1].trim(), "next");
    }

    #[test]
    fn is_dash_rule_rejects_stars() {
        assert!(is_dash_thematic_break("---\n"));
        assert!(is_dash_thematic_break("  - - -\n"));
        assert!(!is_dash_thematic_break("***\n"));
        assert!(!is_dash_thematic_break("___\n"));
    }

    #[test]
    fn unclosed_fields_fence_is_syntax() {
        let err = parse("```yaml\nfield1: foo\n", 2).expect_err("unclosed");
        assert_eq!(err.kind(), crate::ErrorKind::Syntax);
        assert_eq!(err.offset(), Some(0));
        assert!(err.to_string().contains("unclosed fence"), "{err}");
    }

    #[test]
    fn unclosed_body_fence_is_syntax() {
        let input = "```yaml\nk: 1\n```\nsee\n```text\n---\n";
        let err = parse(input, 2).expect_err("unclosed body fence");
        assert_eq!(err.kind(), crate::ErrorKind::Syntax);
        assert!(err.offset().is_some());
    }

    /// Bare YAML whose first key is a list: CommonMark reads the rest of the
    /// mapping as that list item's content, so a fence inside a nested block
    /// scalar is indented past three spaces on the raw line and still closed.
    #[cfg(feature = "yaml")]
    const BARE_YAML_WITH_NESTED_FENCE: &str = "artifacts:\n\
- path: a.md\n  kind: doc\n\
claim: Keep one pass.\n\
reason:\n  text: |-\n    Behaviour ships first.\n\n    ```text\n    # BAD\n    ```\n\
status: candidate\n\
---\n\
Body text.\n";

    #[cfg(feature = "yaml")]
    #[test]
    fn closed_fence_inside_a_container_is_not_unclosed() {
        let doc = parse_ok(BARE_YAML_WITH_NESTED_FENCE);
        let Fields::Bare { sniff, inner, .. } = &doc.fields else {
            panic!("expected bare fields, got {doc:?}");
        };
        assert_eq!(*sniff, Sniff::Yaml);
        assert!(
            inner.contains("    ```text\n    # BAD\n    ```\n"),
            "{inner}"
        );
        assert!(inner.ends_with("status: candidate\n"), "{inner}");
        assert_eq!(doc.body, vec!["Body text.\n".to_owned()]);
    }

    #[test]
    fn closed_fences_in_a_list_and_a_quote_are_not_unclosed() {
        let input =
            "```yaml\nk: 1\n```\n- item\n\n  ```text\n  code\n  ```\n\n> ```\n> quoted\n> ```\n";
        let doc = parse_ok(input);
        assert_eq!(doc.body.len(), 1, "{doc:?}");
    }

    #[test]
    fn a_fence_in_a_list_left_open_is_still_unclosed() {
        let input = "```yaml\nk: 1\n```\n- item\n\n  ```text\n  code\n";
        let err = parse(input, 2).expect_err("unclosed fence in a list item");
        assert_eq!(err.kind(), crate::ErrorKind::Syntax);
    }

    #[test]
    fn dash_in_blockquote_does_not_split() {
        let input = "```yaml\nk: 1\n```\n> ---\nstill section one\n---\nsection two\n";
        let doc = parse_ok(input);
        assert_eq!(doc.body.len(), 2);
        assert!(doc.body[0].contains("---"), "{:?}", doc.body[0]);
        assert!(
            doc.body[0].contains("still section one"),
            "{:?}",
            doc.body[0]
        );
        assert_eq!(doc.body[1].trim(), "section two");
    }
}
