# Roadmap

Version 0.1, 17 September 2026.

Work order so the crate `serde_markdown` implements [DESIGN.md](DESIGN.md).
The mapping is already locked: optional first fields block (fenced or
bare YAML/JSON/TOML), then body sections split on top-level `---`,
body fields marked explicitly. M0–M2 are done. This file remains the
work order for M3–M14.

This is not a 1.0 release plan. v1 is 0.x. Compatible additions bump
the patch; breaking API changes bump the minor ([CHANGELOG.md](CHANGELOG.md)).

The first implementation is the crate `serde_markdown` (plus
`serde_markdown_derive` and generated buffa types). Other language
bindings are out of the M stages. Consumers of the crate are not a
stage here. DESIGN.md §12 stays closed. Do not reopen those rows as
new stages; M boxes implement them.

---

## 1. How to track progress

**Stop after every stage.** The owner reviews that stage's result before
the next stage starts. Spec stages (M0): the review artefact is the
document diff. Implementation stages (M1–M14): the review artefact is the
code + test diff, and `bazel test //...` is green. Do not start the next M
stage from a verbal claim that the previous one is ready.

Tick a box only when the cited *Check:* exists (implementation) or the
owner has accepted the document diff (spec). Do not tick from a verbal
claim.

| Mark  | Meaning |
| ----- | ------- |
| `[ ]` | Not done |
| `[x]` | Spec: owner accepted the diff. Code: the check lives in the crate tests |

Each stage has **Status:** `not started` | `in progress` | `done`. Set
`done` only when every box in that stage is `[x]`. For M1–M14, Clippy is
clean and `bazel test //...` has exited 0 on that tree.

If DESIGN.md has not yet decided a path, a field option, a fence
language, or an error kind, stop. Amend DESIGN.md; do not invent a
silent default in `src/`. Do not implement M2 against this charter
alone.

Order: M0 → M1 → … → M14. Do not interleave a later M stage with an
earlier one. M4 may add the derive crate during M3 only if ser/de does
not use it yet; wiring is M5.

---

## 2. Dashboard

| Stage | Status | What closes |
| ----- | ------ | ----------- |
| [M0](#m0-charter) | done | This file; DESIGN.md is the spec |
| [M1](#m1-skeleton) | done | Bazel/Buf, `options.proto`, testdata, stub crate |
| [M2](#m2-parse) | done | `parse.rs`; CommonMark split; sniff; prefix |
| [M3](#m3-error-format-markdown-trait) | done | `Error`, `Format`, `FieldsLayout`, `Markdown` |
| [M4](#m4-derive) | done | `#[derive(Markdown)]` / `#[markdown(body)]` |
| [M5](#m5-yaml-fenced) | done | `to_string` / `from_str` YAML fenced, hand-written |
| [M6](#m6-document-shapes) | not started | Bare, unlabeled, prefix, body-only, fence-not-first |
| [M7](#m7-json-toml-bare-serialize) | not started | JSON/TOML features; `FieldsLayout::Bare` |
| [M8](#m8-option-and-presence) | not started | Trailing vs middle `None`; `Some("")` |
| [M9](#m9-nested-structured-names-whitespace) | not started | Nested fields, structured body, rename, whitespace |
| [M10](#m10-wkt) | not started | WKT in fence and body via `buffa-types` |
| [M11](#m11-buffa-annotate) | not started | `annotate_markdown_body`; generated messages |
| [M12](#m12-io-and-features) | not started | `to_vec` / `from_slice` / writer; feature matrix |
| [M13](#m13-documents-once) | not started | DESIGN, README, CHANGELOG, this dashboard |
| [M14](#m14-acceptance) | not started | Golden round-trips; `bazel test //...` |

---

## 3. Locked decisions

M2–M14 implement these. They live in DESIGN.md §12. Do not reopen them
in a later M stage without amending DESIGN.md §12 and this section
first.

1. **Body fields are marked, any type.** Rust: `#[markdown(body)]`.
   Protobuf package `markdown`: `(markdown.body) = true` (proto3 and
   editions). WKT via buffa serde. Not inferred from `Vec<u8>`.
2. **YAML crate is `yaml_serde`.** Not `serde_yaml`, not `serde_yml`.
3. **Serialize default is YAML**, fenced ` ```yaml `. JSON/TOML only if
   `to_string_with_format`.
4. **Fields are first only** (after skipping leading whitespace and
   `---`): a fenced yaml/json/toml code block, or **bare** YAML/JSON/TOML
   mapping before the first `---`. Unlabeled fence / bare slice → sniff.
   Not a mapping → body-only. A later fence is body.
5. **Markdown-aware split** via `pulldown-cmark` (not comrak).
6. **`Option` / proto presence in v1**, trailing-absent omitted,
   middle-absent keeps an empty slot (DESIGN.md §4.4).
7. **TOML** fence language in v1.
8. **Split on top-level dash thematic breaks only**; ignore `***` /
   `___` and any `---` inside containers/fences.
9. **Protobuf option number `20260917`.**
10. **`BODY_FIELDS` uses buffa proto3 JSON names** (`published_at` →
    `publishedAt`).
11. **Handwritten `Error` / `ErrorKind`.** No `thiserror`, no `anyhow`,
    no proto error envelope. Callers match `kind()`. `Syntax` has a
    byte offset. Decoder and IO failures are `source()`.

Open questions in DESIGN.md: none.

---

## Examples

Target usage after M14. Both produce the same document (DESIGN.md §2.1).
These are not extra M stages.

### Rust struct

```rust
use serde::{Deserialize, Serialize};
use serde_markdown::{from_str, to_string, Markdown};

#[derive(Debug, PartialEq, Serialize, Deserialize, Markdown)]
struct Page {
    field1: String,
    field2: String,
    field3: i32,
    #[markdown(body)]
    text1: String,
    #[markdown(body)]
    text2: String,
}

let page = Page {
    field1: "foo".into(),
    field2: "bar".into(),
    field3: 1,
    text1: "Text1 bla bla bla".into(),
    text2: "Text2 bal bla bla".into(),
};
let md = to_string(&page)?;
let back: Page = from_str(&md)?;
assert_eq!(page, back);
```

````markdown
```yaml
field1: foo
field2: bar
field3: 1
```
Text1 bla bla bla
---
Text2 bal bla bla
````

`#[derive(Markdown)]` is M4. `to_string` / `from_str` are M5.

### Protobuf

```protobuf
syntax = "proto3";
package example;

import "markdown/options.proto";

message Page {
  string field1 = 1;
  string field2 = 2;
  int32 field3 = 3;
  string text1 = 4 [(markdown.body) = true];
  string text2 = 5 [(markdown.body) = true];
}
```

```rust
// build.rs — M11
buffa_build::Config::new()
    .files(&["proto/page.proto"])
    .includes(&["proto/", serde_markdown::PROTO_INCLUDE])
    .generate_json(true)
    .apply(serde_markdown::buffa::annotate_markdown_body)
    .compile()?;

// after generate
use serde_markdown::{from_str, to_string};

let page = example::Page { /* field1, field2, field3, text1, text2 */ };
let md = to_string(&page)?;
let back: example::Page = from_str(&md)?;
```

Same Markdown as the Rust example. `BODY_FIELDS` is `["text1", "text2"]`
(proto3 JSON names; one-word identifiers stay unchanged). Owned generated
messages only; views are out of M0–M14.

---

## 4. Out of M0–M14

These must not block a stage in §2. Writes in M2–M14 must not depend on
them.

- Pandoc / Jekyll `---` YAML `---` front matter as an alternate syntax
  (DESIGN.md §10).
- Splitting body by ATX headings (`## Intro`) instead of `---`.
- Streaming huge files.
- Preserving YAML comments, anchors, or key style.
- `no_std`.
- Generated `Markdown` impl for buffa **views**.
- Forking `buffa-codegen`.
- Treating proto `bytes` body fields as raw UTF-8 (they stay proto JSON
  base64 unless DESIGN.md adds an opt-in later).
- `thiserror` / `anyhow` / a proto error envelope for crate `Error`
  (DESIGN.md §7 / §12).
- `Any` packing with a `TypeRegistry` (DESIGN.md §4.3 / §9: later).
- crates.io publish; crate version number for a 1.0.
- Other language bindings.

---

## M0. Charter

**Status:** done

**Codes:** none yet. This stage does not change `src/`.

Record that DESIGN.md is the spec and point the package at this work
order. Do not implement parse or ser/de.

- [x] `ROADMAP.md` exists and §3 matches DESIGN.md §12.
- [x] README names this file next to DESIGN.md.
- [x] DESIGN.md §11 defers stage order to this file.
- [x] Owner has reviewed this file. Next stage is M2 (M1 is already
      done), not M5.

**Review:** this document. Stop.

---

## M1. Skeleton

**Status:** done

**Codes:** DESIGN.md §5.1, §6.4–6.5, §9 (fixtures on disk).

Close only after M0 exists. The tree already compiled before this file.
Do not reopen Bazel/Buf shape in a later M stage without amending
DESIGN.md §6.5.

- [x] Workspace crates `serde_markdown` and `serde-markdown-generated`
      build (`cargo test -p serde_markdown`, `cargo test -p serde-markdown-generated`).
- [x] `proto/markdown/options.proto` is proto3, package `markdown`,
      field `body = 20260917` on `FieldOptions`.
- [x] `bazel run //proto/markdown:generate` writes
      `rust/generated/markdown/`; `bazel test //proto/markdown:generate_test`
      and `//proto/markdown:lint` pass.
- [x] Fixture messages live in `proto/markdown/testdata/` (not the public
      `markdown` API). Golden Markdown lives in
      `rust/serde_markdown/testdata/markdown/`. Hand-written structs and
      constructors live in `rust/serde_markdown/src/testdata/`.
- [x] `bazel test //rust:lint` `//bazel:lint` `//bazel:markdown` pass.
- [x] Public API is still a stub (`DESIGN` const). No `from_str` /
      `to_string` yet.

**Review:** skeleton. Stop. Do not start parse in this stage.

---

## M2. Parse

**Status:** done

**Codes:** DESIGN.md §2.5–2.7, §6 (`parse.rs`).

Close only after M1. Implement the CommonMark split only. Do not
deserialize YAML/JSON/TOML in this stage.

Recommended files: `src/parse.rs`. Tests may `include_str!` the goldens
already in `testdata/markdown/`.

- [x] `OffsetIter` over `pulldown-cmark` events. Track block nesting
      (blockquote, list, item, table, footnote).
- [x] Leading Unicode whitespace and top-level dash `---` are discarded
      (`page.leading_prefix.md`). They are not a body section.
- [x] First top-level fenced code block whose info string (trimmed,
      case-insensitive, first token) is `yaml` / `yml` / `json` /
      `toml` or empty is the fields fence. Any other info string is
      not fields (`page.fence_not_first.md`).
- [x] Unlabeled first fence and a bare first slice report a sniff
      hint: `{` / `[` → JSON; TOML-shaped first line → TOML; else YAML
      (DESIGN.md §2.6). Empty fence inner text → YAML. Do not run the
      YAML/JSON/TOML parsers yet.
- [x] If there is no fields fence, the text until the next top-level
      dash `---` (or EOF) is a bare-fields candidate. Mapping vs not is
      M6; this stage only yields the slice.
- [x] Body splits on remaining top-level dash `---` only.
      `page.split.fence.md`: `---` inside a fence does not split.
      `page.split.stars.md`: `***` / `___` do not split.
      `page.split.list.md`: `---` inside a list item does not split.
- [x] Public parse type is crate-private unless rustdoc in M12 needs it.
- [x] `bazel test //...` green. Owner has reviewed the diff.

**Review:** split tests. Stop. Do not call `yaml_serde` in this stage.

---

## M3. Error, Format, Markdown trait

**Status:** done

**Codes:** DESIGN.md §5 (`Format`, `FieldsLayout`, `Markdown`), §7.

Close only after M2.

- [x] Handwritten `Error` / `ErrorKind` in `error.rs` (no `thiserror`,
      no `anyhow`, not a proto message). Implements `std::error::Error`,
      `Display`, `serde::{ser,de}::Error` by hand. Kinds: `Syntax`,
      `FrontMatter`, `Body`, `Type`, `FormatDisabled`, `Io`. Callers
      match `kind()`. `Syntax` has a byte offset. `FrontMatter` and
      `Io` (and structured-body decoder failures under `Body`) set
      `source()`.
- [x] `Format` is `Yaml` (default) / `Json` / `Toml`.
      `FieldsLayout` is `Fenced` (default) / `Bare`.
- [x] `Markdown` trait: `const BODY_FIELDS: &'static [&'static str]`.
      Testdata hand-written types implement it (lists already in
      `src/testdata/types.rs`).
- [x] Root that is not a named struct is `Type` (tests may wait for
      M5 if they need `to_string`).
- [x] Module layout matches DESIGN.md §6.4 for the files this stage
      adds (`error.rs`, `format.rs`, `markdown.rs`). `ser.rs` / `de.rs`
      may exist as stubs.
- [x] `bazel test //...` green. Owner has reviewed the diff.

**Review:** public types without round-trip. Stop.

---

## M4. Derive

**Status:** done

**Codes:** DESIGN.md §3.1.

Close only after M3. Ser/de may still use a manual `Markdown` impl until
M5. Do not parse proto options here (M11).

- [x] Workspace crate `serde_markdown_derive` with
      `#[derive(Markdown)]` and `#[markdown(body)]`.
- [x] Emitted `BODY_FIELDS` uses the **Serde field name** (after
      `rename`). A compile-fail or unit test covers `publishedAt` vs
      `published_at` (`types::JsonNames`).
- [x] Feature `derive` on `serde_markdown` (DESIGN.md §5.1). Default
      features may include it; Bazel still links it.
- [x] `bazel test //...` green. Owner has reviewed the diff.

**Review:** derive crate. Stop.

---

## M5. YAML fenced

**Status:** done

**Codes:** DESIGN.md §2.1, §4.1–4.2, §5 `to_string` / `from_str`, §6.3,
§8.

Close only after M4. Hand-written structs only. Default serialize:
`Format::Yaml` + `FieldsLayout::Fenced`.

- [x] `to_string` / `from_str` exist. `from_str(to_string(x)) == x` for
      `types::Page` without `published` (`page.fenced.yaml.md` as
      deserialize input; serialize need not be byte-identical).
- [x] Fence language on serialize is labeled `yaml`. One newline after
      the closing fence; separator `\n---\n`; trailing newline at EOF;
      no leading blank line; no trailing `---` after the last section
      (DESIGN.md §8).
- [x] Body `String` sections are raw (no extra quotes). Front-matter
      keys are Serde names. `yml` as a deserialize tag is accepted
      (`page.fenced.yml.md`).
- [x] Fields-only document omits body (`fields_only.fenced.yaml.md`).
      All-body with a fence is not this stage.
- [x] `bazel test //...` green. Owner has reviewed the diff.

**Review:** YAML fenced round-trip. Stop.

---

## M6. Document shapes

**Status:** not started

**Codes:** DESIGN.md §2.4–2.6.

Close only after M5. Deserialize the rest of the Page-shaped goldens.
Serialize may still be fenced YAML.

- [ ] Bare YAML/JSON/TOML first slice that is a mapping is fields
      (`page.bare.yaml.md`, `page.bare.json.md`, `page.bare.toml.md`).
- [ ] Unlabeled first fence sniffs (`page.unlabeled.yaml.md`,
      `.json.md`, `.toml.md`). Sniffed JSON/TOML parse failure is
      `FrontMatter`, not a fallback to YAML.
- [ ] Leading whitespace / `---` prefix is ignored
      (`page.leading_prefix.md`).
- [ ] First slice that is not a mapping is body-only
      (`body_only.two_sections.md`). Prose then a later ` ```yaml ` is
      body (`page.fence_not_first.md`): that fence is not fields.
- [ ] `page.split.*` round-trip as one first body section plus
      appendix (split behaviour from M2, now through `from_str`).
- [ ] `bazel test //...` green. Owner has reviewed the diff.

**Review:** document shapes. Stop.

---

## M7. JSON, TOML, bare serialize

**Status:** not started

**Codes:** DESIGN.md §2.2–2.4, §5 `to_string_with_format` /
`to_string_with`, §5.1 feature errors.

Close only after M6.

- [ ] `to_string_with_format(..., Format::Json)` writes a labeled
      ` ```json ` fence (pretty, 2 spaces). Same for `Toml` and
      `toml::to_string_pretty`. Goldens `page.fenced.json.md` /
      `page.fenced.toml.md` deserialize.
- [ ] `FieldsLayout::Bare` writes DESIGN.md §2.4 (no fence; `---`
      before the first body section). `to_string` still fenced YAML.
- [ ] `from_str` of ` ```json ` without the `json` feature is
      `FormatDisabled`. Same for yaml/toml. A test crate or
      `trybuild`/feature-gated test covers at least one disabled
      format.
- [ ] `bazel test //...` green. Owner has reviewed the diff.

**Review:** formats and bare serialize. Stop.

---

## M8. Option and presence

**Status:** not started

**Codes:** DESIGN.md §4.4.

Close only after M7. Use `types::OptionalBody` and
`optional.*.md`.

- [ ] `Some("a"), None, Some("c")` ↔ `optional.middle_none.md`.
- [ ] Trailing absent omitted (`optional.trailing_none.md`).
- [ ] Leading absent is an empty first section
      (`optional.leading_none.md`).
- [ ] `Some("")` is a section whose content is the two characters `""`
      (`optional.empty_string.md`).
- [ ] Extra trailing sections → `Body` (too many). Missing section for
      a required body field → `Body`. Empty section for a required
      structured type → `Body` (may wait for M9 if no fixture yet).
- [ ] `bazel test //...` green. Owner has reviewed the diff.

**Review:** presence matrix. Stop.

---

## M9. Nested, structured, names, whitespace

**Status:** not started

**Codes:** DESIGN.md §4.1–4.2, §8, §9 (nested / rename / whitespace).

Close only after M8.

- [ ] Nested front matter (`nested.fenced.yaml.md`,
      `types::NestedFields`).
- [ ] Structured body section is fence-format dump of the object
      (`structured.fenced.yaml.md`, `types::Article`).
- [ ] `#[serde(rename)]` / proto3 JSON names: Markdown keys are
      `publishedAt` / `bodyNote` (`names.fenced.yaml.md`).
- [ ] Do not trim section interiors. Trailing spaces and internal
      blank lines survive (`whitespace.unicode.md`). Unicode in body
      is UTF-8.
- [ ] `bazel test //...` green. Owner has reviewed the diff.

**Review:** nested and whitespace. Stop.

---

## M10. WKT

**Status:** not started

**Codes:** DESIGN.md §4.3.

Close only after M9. Hand-written structs with `buffa_types` and the
generated `WellKnown` message as a deserialize target if the `Markdown`
impl is still manual. Generated `Markdown` impl is M11.

- [ ] `well_known.fenced.yaml.md` round-trips Timestamp, Duration,
      Empty, Struct, FieldMask, Int32Value in the fence; Timestamp and
      Duration as body sections (raw RFC 3339 / `"1.5s"` form).
- [ ] Do not hand-roll RFC 3339. Mapping is buffa proto3 JSON serde.
- [ ] `page.published.yaml.md` sets `published` on `types::Page` /
      generated `Page`.
- [ ] `Any` is not required. Document that callers who need `Any`
      install a registry (DESIGN.md §4.3); no test in this stage.
- [ ] `bazel test //...` green. Owner has reviewed the diff.

**Review:** WKT. Stop.

---

## M11. Buffa annotate

**Status:** not started

**Codes:** DESIGN.md §3.2 (buffa integration).

Close only after M10. `options.proto` already exists (M1). This stage
is the helper and generated-message round-trips.

- [ ] Feature `buffa`: `annotate_markdown_body` walks a
      `FileDescriptorSet`, finds `(markdown.body) = true`, injects
      `#[derive(Markdown)]` / `#[markdown(body)]` via buffa
      `message_attribute` / `field_attribute`. No fork of
      `buffa-codegen`.
- [ ] `BODY_FIELDS` on generated types uses proto3 JSON names
      (`occurredAt`, `bodyNote`).
- [ ] Generated `Page`, `Proto3Page`, `JsonNames`, `OptionalBody`,
      `Article`, `WellKnown` round-trip the matching goldens
      (`from_str` / `to_string`). proto3 `optional` body works.
- [ ] Views (`PageView`) are not `Markdown` (DESIGN.md §10).
- [ ] `bazel test //...` green. Owner has reviewed the diff.

**Review:** generated messages. Stop.

---

## M12. IO and features

**Status:** not started

**Codes:** DESIGN.md §5 (`to_vec`, `to_writer`, `from_slice`,
`from_reader`), §7 `Io` / `FormatDisabled`, §8 UTF-8.

Close only after M11.

- [ ] `to_vec` / `to_writer` / `from_slice` / `from_reader` match
      `to_string` / `from_str`. `from_slice` fails on invalid UTF-8.
- [ ] rustdoc on the public surface points at DESIGN.md for the
      mapping; examples use `Page` or a 5-line struct.
- [ ] Feature matrix: default features include yaml/json/toml as in
      DESIGN.md §5.1 (derive/buffa as decided in M4/M11). Disabled
      format still `FormatDisabled`.
- [ ] `bazel test //...` green. Owner has reviewed the diff.

**Review:** IO and features. Stop.

---

## M13. Documents once

**Status:** not started

Close only after M12 and green tests. One pass over documents that still
contradict the code. Do not polish twice.

- [ ] DESIGN.md, README, this file's dashboard statuses, and module
      rustdoc match shipped behaviour (function names, feature names,
      error kinds, defaults).
- [ ] DESIGN.md §11 is a pointer to this file, not a second checklist.
- [ ] CHANGELOG `## [Unreleased]` states what M2–M12 shipped when this
      work is committed; this box is the note, not a separate
      changelog-only commit.
- [ ] Owner has reviewed the docs diff.

**Review:** document consistency. Stop.

---

## M14. Acceptance

**Status:** not started

Close only after M13.

- [ ] Every golden in `testdata::goldens::ALL` that is a supported
      round-trip deserializes to the matching `values::*` constructor
      (or an explicit skip list in the test for serialize-only /
      parse-only files, named in rustdoc).
- [ ] `from_str(to_string(x)) == x` for the hand-written and generated
      constructors in `src/testdata/values.rs` that are complete values
      (not trailing-`None` serialize shapes already covered in M8).
- [ ] `page.split.*` still do not split on inner `---` / `***` / lists.
- [ ] `bazel test //...` green (includes `//rust:lint`,
      `//proto/markdown:lint`, `//proto/markdown:generate_test`,
      `//bazel:markdown`). Owner has reviewed the fixture tests.

**Review:** acceptance. After this, `ROADMAP.md` is done for the crate.

---

## 5. What this plan does not decide

- Crate version number for a crates.io release (CHANGELOG / 0.x rules
  at publish).
- Exact `Error` `Display` wording. M3 picks strings and rustdoc.
- Whether the default feature set includes `derive` and `buffa` on day
  one. DESIGN.md §5.1 lists them; M4 and M11 may ship them off-default
  if Bazel CI still links them. Record the choice in DESIGN.md if it
  differs.
- Whether `annotate_markdown_body` is public in v1 or
  `#[doc(hidden)]` until a consumer build.rs exists. M11 picks and
  rustdocs.

---

## 6. Open questions

None that block M2. DESIGN.md §12 is closed. Remaining items in §5 are
implementation choices for later M stages. If the owner rejects a locked
decision in §3, amend DESIGN.md §12 and this file before that later
stage.

---

## 7. Consumers

Out of this plan. After M14 a consumer adds `#[derive(Markdown)]` (or
imports `options.proto` and runs `annotate_markdown_body`) and calls
`from_str` / `to_string`. See [Examples](#examples). How any one
consumer wires that into their tree is not an M stage.
