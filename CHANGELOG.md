# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
While the major version is 0, compatible additions bump the patch; breaking
API changes bump the minor.

## [Unreleased]

## [0.2.3] - 2026-10-02

### Fixed

- A fenced code block inside a list item or a block quote was reported as an
  unclosed fence (`Syntax`) although its closing line was there: the check
  allowed at most three spaces before the closer on the raw line, and no `>`
  marker. The closer is now judged against the opener's column, after the
  container prefix. Bare YAML fields whose first key is a list read the rest
  of the mapping as that list item, so a multi-line string holding a fenced
  block was written by `FieldsLayout::Bare` and then refused on read.

## [0.2.2] - 2026-10-01

### Changed

- Updated Rust toolchain and minimum supported Rust version from 1.98.1 to
  1.99.0 (Cargo `rust-version` and Bazel `RUST_VERSION`).
- Added `rust-toolchain.toml` to pin the Rust toolchain for Cargo builds.

## [0.2.1] - 2026-09-27

### Changed

- Updated all `bazel_utils_*` modules to `v0.2.11`.
- Updated `bazel_utils_*` git remotes to use sonalect forks.
- Protobuf well-known types now sourced locally via `proto/third_party` instead
  of remote Buf dependency.
- Updated `buf.yaml` to remove remote wellknowntypes dependency.

## [0.2.0] - 2026-09-27

### Changed

- Reorganized workspace structure: renamed `rust/serde_markdown` →
  `rust/markdown`, `rust/serde_markdown_derive` → `rust/markdown_derive`, and
  consolidated generated code into a new `rust/proto` module (replacing
  `rust/generated`).
- Updated all `bazel_utils_*` modules to `v0.2.10`.
- Updated `protobuf` to `v36.2`.

## [0.1.1] - 2026-09-19

### Changed

- Bazel crate universe hub is `serde_markdown_cargo`, so another module
  that names its hub `cargo` can depend on this repo as a `bazel_dep`.
  Transitive `bazel_dep` uses a crate_universe `lockfile`.
- `bazel_utils_*` modules `v0.2.9`.
- `serde_markdown_derive` uses `syn` 3.0.6.

## [0.1.0] - 2026-09-18

### Added

- Design for a Serde format crate that maps a root struct to a Markdown
  document: optional first fenced fields block (YAML by default; JSON or TOML
  when requested), then body sections split on top-level `---` (see
  [DESIGN.md](DESIGN.md)).
- Field marking: Rust `#[markdown(body)]`; protobuf `(markdown.body) = true`
  (proto3 and editions). Body field type is unrestricted. `BODY_FIELDS` uses
  buffa proto3 JSON names (`published_at` → `publishedAt`).
- Protobuf option `markdown.body` on `google.protobuf.FieldOptions`, field
  number `20260917` (`proto/markdown/options.proto`, proto3; usable from
  proto3, Edition 2023, and Edition 2024).
- Fixture `markdown.testdata.Page`
  (`proto/markdown/testdata/page.proto`) with well-known `Timestamp` and
  two body strings, compiled by buffa into `rust/generated`. Not part of
  the public `markdown` proto API.
- Testdata set covering DESIGN.md shapes: proto messages in
  `proto/markdown/testdata/`, golden Markdown in
  `rust/serde_markdown/testdata/markdown/`, and matching hand-written
  Rust structs in `rust/serde_markdown/src/testdata/`.
- Crate-private CommonMark split (`rust/serde_markdown/src/parse.rs`):
  fields fence or bare first slice, sniff hint without YAML/JSON/TOML
  parsers, body sections on top-level dash `---` (including setext H2
  underlines; not `***` / `___`, not `---` inside fences or lists).
- Implementation work order [`ROADMAP.md`](ROADMAP.md) (M0–M15).
- Two runnable example crates: a hand-written `Page` struct and a protobuf
  `Page` generated in the example crate, both printing the same fenced YAML
  document (`cargo run -p example-page` / `cargo run -p example-protobuf`).
- Bazel + Buf pipeline: `buf_module`, lint, format,
  prebuilt `protoc-gen-buffa` and `protoc-gen-buffa-packaging` from
  `bazel_utils_protoc` (`protoc.plugin` tags on `buf_generate` PATH),
  `bazel run //proto/markdown:generate`. [`buf.lock`](buf.lock) pins
  well-known types so Buf LSP can resolve `google/protobuf/*.proto`.
- Workspace crates `serde_markdown`, `serde_markdown_derive`, and
  `serde-markdown-generated`.
- Markdown split parser choice: `pulldown-cmark`. Fields may be a first
  fenced block or **bare** YAML/JSON/TOML (no ` ``` `) before the first
  `---`; format is sniffed. Leading whitespace and `---` before fields are
  ignored. A first slice that is not a mapping is body-only.
- Default serialize format YAML (`yaml_serde`) as a **fenced** ` ```yaml `
  block; JSON and TOML via `to_string_with_format`; bare (unfenced) fields
  via `FieldsLayout::Bare`. `Option` / proto presence in body sections from
  v1.
- Public mapping-layer types: handwritten `Error` / `ErrorKind` (match
  `kind()`; `Syntax` carries a byte offset; decoder and I/O failures are
  `source()`), `Format` (`Yaml` default / `Json` / `Toml`),
  `FieldsLayout` (`Fenced` default / `Bare`), and the `Markdown` trait
  (`BODY_FIELDS`).
- `to_string` / `from_str` for hand-written structs: default serialize is a
  labeled `yaml` fence plus raw `String` body sections (`yml` is accepted
  on input). Fields-only documents omit the body. Serialize need not be
  byte-identical to a golden; `from_str(to_string(x)) == x` holds.
- `to_string_with_format` writes a labeled `json` or `toml` fence (pretty
  JSON with 2-space indent; `toml::to_string_pretty`). `to_string_with`
  plus `FieldsLayout::Bare` writes unfenced fields and a `---` before the
  first body section. `to_string` stays fenced YAML. A fence language not
  in compiled features is `FormatDisabled`.
- `from_str` also reads the rest of the Page-shaped documents: bare
  YAML/JSON/TOML mappings, unlabeled first fences (sniffed; a sniffed JSON
  or TOML failure is `FrontMatter`, not a YAML fallback), a discarded
  leading whitespace/`---` prefix, body-only when the first slice is not a
  mapping, and `page.split.*` as one first body section plus appendix. A
  later fenced `yaml` block is body, not fields.
- Body `Option<String>` presence: trailing `None` is omitted, a middle
  `None` keeps an empty section, a leading `None` is an empty first
  section, and `Some("")` is the two characters `""`. Extra sections, a
  missing required body section, and an empty section for a required
  structured field are `Body`.
- Nested front-matter mappings, structured body sections as a fence-format
  dump of the object, Serde `rename` keys (`publishedAt` / `bodyNote`),
  and untrimmed body interiors (trailing spaces, internal blank lines,
  Unicode).
- `#[derive(Markdown)]` and `#[markdown(body)]` via workspace crate
  `serde_markdown_derive`, re-exported behind the `derive` feature (on by
  default). Emitted `BODY_FIELDS` uses Serde field names after `rename`.
- Google well-known types in the fields fence and as body sections via buffa
  proto3 JSON serde (`Timestamp`, `Duration`, `Empty`, `Struct`, `FieldMask`,
  `Int32Value`). Body `Timestamp` / `Duration` are the raw RFC 3339 / `1.5s`
  form. Callers who need protobuf `Any` install a type registry
  (`buffa_types::register_wkt_types`); this crate does not.
- Feature `buffa` (off-default; Bazel enables it): `annotate_markdown_body`
  walks a `FileDescriptorSet` for `(markdown.body) = true` and returns buffa
  `message_attribute` / `field_attribute` pairs (`#[derive(Markdown)]` /
  `#[markdown(body)]`). Public `PROTO_INCLUDE` for `markdown/options.proto`.
  Generated testdata messages (Page, Proto3Page, JsonNames, OptionalBody,
  Article, WellKnown) round-trip the matching goldens. Views are not
  `Markdown`.
- `to_vec` / `to_writer` / `from_slice` / `from_reader` match `to_string` /
  `from_str`. Invalid UTF-8 on `from_slice` is `Syntax` with a byte offset.
  Writer and reader failures are `Io`. A fence language not in compiled
  features remains `FormatDisabled`.
- Golden acceptance walks every document in `testdata::goldens::ALL` (explicit
  parse-only skip for split and fence-not-first files) and round-trips complete
  `values::*` constructors through `from_str(to_string(x))`. A new golden that
  is neither deserialized nor skipped fails the walker.

### Changed

- The protobuf example compiles its `Page` when the crate builds (`build.rs`
  plus `annotate_markdown_body`) instead of checking in Buf
  `write_source_files` stubs. `cargo run -p example-protobuf` needs `protoc`
  (`PROTOC` or PATH). Bazel uses the proto toolchain prebuilt
  (`protoc_prefix`), not `@protobuf//:protoc` (that target compiles from
  source).
- Buf generate/lint/format use the workspace [`buf.yaml`](buf.yaml).
  [`buf.gen.rust.yaml`](buf.gen.rust.yaml) `inputs` keep fixture generate on
  `proto/` only.
- DESIGN.md, README, and this file match the shipped crate: function names
  (`from_str` is `DeserializeOwned`), feature `json = []`, derive path
  `rust/serde_markdown_derive`, error kinds and defaults. DESIGN.md §11 is
  only a pointer to ROADMAP.md (M0–M15).
- `serde_markdown_derive` uses `syn` 3.0.5.
- Buf CLI `v1.73.0`; `bazel_utils_*` modules `v0.2.8`.
- Error contract (DESIGN.md §7 / §12): handwritten `Error` and
  `ErrorKind` like `serde_json` (no `thiserror`, no `anyhow`, no proto
  envelope). Callers match `kind()`; `Syntax` has a byte offset;
  decoder and IO failures are `source()`.

### Removed

- `buf.markdown.yaml`. Generate, lint, and format use the workspace
  [`buf.yaml`](buf.yaml).

### Fixed

- Unclosed fenced code blocks fail with `Syntax` and a byte offset instead of
  being treated as fields or body.
- Body `Vec<u8>` (and `serialize_bytes` buffers) write raw UTF-8 Markdown;
  invalid UTF-8 on serialize is `Body`. They are no longer dumped as a
  YAML/JSON/TOML array of numbers.
- The workspace `buf.yaml` lists the protobuf example proto directory so Buf
  LSP can resolve `markdown/options.proto`.

## Links

- [Unreleased]
- [0.2.3]
- [0.2.2]
- [0.2.1]
- [0.2.0]
- [0.1.1]
- [0.1.0]

[Unreleased]: https://github.com/sonalect/serde_markdown/compare/v0.2.3...HEAD
[0.2.3]: https://github.com/sonalect/serde_markdown/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/sonalect/serde_markdown/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/sonalect/serde_markdown/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/sonalect/serde_markdown/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/sonalect/serde_markdown/releases/tag/v0.1.1
[0.1.0]: https://github.com/sonalect/serde_markdown/releases/tag/v0.1.0
