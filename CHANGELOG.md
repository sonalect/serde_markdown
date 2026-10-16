# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
While the major version is 0, compatible additions bump the patch; breaking
API changes bump the minor.

## [Unreleased]

### Added

- Ready for crates.io: keywords, categories, and the license text in both
  packages, a README for `serde_markdown_derive`, and docs.rs builds
  `serde_markdown` with every feature (the `buffa` and `tokio` modules).
  `serde_markdown` requires its own `serde_markdown_derive` by exact
  version.
- CI builds and tests `serde_markdown` and `serde_markdown_derive` with
  the minimum Rust version, read from `rust-version`.

### Changed

- The minimum Rust version (`rust-version`) is 1.88 (it was 1.99.0), the
  oldest rustc that builds and tests the published crates.
- The workspace takes its dependencies as caret ranges instead of exact
  pins (`serde = "1.0.229"`, was `"=1.0.229"`), so a consumer is not held to
  one version of serde, buffa, tokio, or the format crates. `Cargo.lock`
  still fixes the versions this repository builds and tests; Renovate moves
  it within the ranges.
- The README takes `serde_markdown` from crates.io
  (`serde_markdown = "0.5"`) and links to GitHub by absolute URLs.

- Dependencies: at least `serde_json` 1.0.154, `syn` 3.0.9, and `tokio`
  1.53.3.
- Tooling: `bazel_utils_*` v0.4.0, Buf CLI v1.74.0, and
  `bazel-contrib/setup-bazel` 0.19.1 in CI. Buildifier, markdownlint, buf
  lint, and cargo audit now also run on Windows (amd64 and arm64), so every
  test but the doc tests runs on all six CI platforms; rules_rust cannot link
  doc tests on Windows (LNK1181).

## [0.5.0] - 2026-10-16

### Added

- Bazel test targets for the builds with only `tokio` and only `buffa`
  (`serde_markdown_tokio_only`, `serde_markdown_buffa_only`), next to the
  all-features, `no_json`, `no_toml`, `no_yaml`, and `minimal` builds.

### Changed

- **Breaking:** a key of the fields block that is named like a body field is
  `ErrorKind::Type` on read (the message names the key). Before, the key was
  dropped without a word, even from a type with `deny_unknown_fields`.
- **Breaking:** a number that is not finite (`.inf`, `.nan`, `inf`, `nan`) is
  an error on read: `ErrorKind::FrontMatter` in the fields block,
  `ErrorKind::Body` in a structured body section. Before, it read as an
  absent value (`None`). The writer refuses such a number in every language
  (JSON used to write `null`): `ErrorKind::FrontMatter` in the fields block,
  `ErrorKind::Body` in a structured body section.
- **Breaking:** the writer refuses a fields block that a reader would not
  read back as written: `ErrorKind::FrontMatter` with no source. In the bare
  layout this is a string holding a `---`, fence, or HTML comment line, or
  any value whose encoding the reader would split or take for body; the
  message says to write it with `FieldsLayout::Fenced`.
- **Breaking:** a `Markdown::BODY_FIELDS` name that the value never
  serializes (not even skipped) is `ErrorKind::Type` at write. A manual impl
  that named a missing field silently lost its section before.
- **Breaking:** `#[derive(Markdown)]` rejects a body field that carries
  `#[serde(skip)]`, `skip_serializing`, or `skip_deserializing` (compile
  error); use `skip_serializing_if` and `default` for an optional body field.
- **Breaking:** the constructors `Error::syntax`, `Error::front_matter`,
  `Error::body`, `Error::body_source`, `Error::type_error`,
  `Error::format_disabled`, and `Error::io` are no longer public. Errors are
  made by the crate; read them with `kind()`, `offset()`, `Display`, and
  `source()`. To make one in your own `Serialize` or `Deserialize` code, use
  serde's `Error::custom`, which gives `ErrorKind::Type`. `?` on an
  `std::io::Error` still converts, through `From<io::Error>`.
- **Breaking:** `ErrorKind` is `#[non_exhaustive]`. A `match` on it needs a
  `_` arm, so a kind added later does not break your build.
- Format sniffing is strict. `{` is JSON. TOML is chosen only for a table
  header (`[key]`, `[[key]]`) or a `key = value` first line whose value
  starts like a TOML value. A bare first slice that starts with `[` and is
  not a table header (a Markdown link, a list) is body, with no decoder
  run; an unlabeled fence starting with `[` is still JSON. A body-only
  document starting with `[text](url) more` or `E = mc² is famous` no longer
  fails with `ErrorKind::FrontMatter`.
- A map key that is not a string (`BTreeMap<u32, _>`, `i64`, `bool`) reads
  in every language. Reading goes through `serde_json::Value` as the
  deserializer.
- A `Vec<u8>` body is no longer decoded as a document to find out whether it
  is a list, unless it starts like one (`[` or `-` after whitespace and `#`
  comment lines). A body-only document is no longer decoded as YAML unless
  its first line can start a mapping (it has a `:`, or starts with `?`, `{`,
  `!`, `&`, `*`, or `%`). Results are the same; a 6 MB body is read and
  written about twice as fast.
- The CommonMark parser stops after the last separator that splits the
  earlier body fields, so the verbatim last body field is not parsed. A 6 MB
  last field reads about three times as fast.
- `serde_markdown::PROTO_INCLUDE` is for Cargo builds; under Bazel a build
  script takes the directory from the annotated filegroup (README, "The
  option in Bazel").
- DESIGN.md, README, and ROADMAP.md match the code (sniffing, the fields
  fence, errors, tests, M15 and M16 done).

### Removed

- **Breaking:** `serde_markdown::DESIGN`, the constant holding the file name
  `DESIGN.md`. It had no use in code.

### Fixed

- A fenced fields block whose encoded text has a line of backticks (a YAML
  or TOML string holding a fence) closed early and read back with no error
  as different data. The fence is now longer than any such run.
- `FieldsLayout::Bare` with TOML and every field absent wrote a stray
  `---` and read back the body shifted. An empty bare block is now no block.
- A bare TOML block that starts with a table (`[meta]`) was sniffed as JSON
  and failed to read back.
- A newtype body field (`struct Title(String)`) was read through the YAML
  parser (`"# Title\n\n- item"` came back as `"item"`), and a `char` body
  field `#`, `-`, or `~` failed. Both are now raw section text; a newtype
  around a struct or a number is still read in the fence language.
- A body-only document that needs an explicit empty fields block, written by
  a build without that language, wrote a block its own reader refuses. It
  is now `ErrorKind::FormatDisabled` at write.

## [0.4.1] - 2026-10-12

### Changed

- Dependencies: `syn` =3.0.7, `toml` =1.1.8.

## [0.4.0] - 2026-10-07

### Added

- Feature `tokio` (off by default) adds `serde_markdown::tokio`, the async
  twin of every function under the same name: `to_string`,
  `to_string_with_format`, `to_string_with`, `to_vec`, `to_writer` (tokio
  `AsyncWrite`), `from_str`, `from_slice`, `from_reader` (tokio
  `AsyncRead`). It writes and reads the same documents as the sync form. It
  is native: the same steps, with a cooperative yield between them when the
  task's budget is spent; no thread and no `spawn_blocking`, so an async
  caller awaits it as it is.
- The runnable example `example-tokio` (`rust/examples/tokio`): the async
  form on a tokio runtime, with a file round-trip over `tokio::fs` and
  pages rendered in parallel tasks.

### Changed

- The README is rewritten for users of the crate: how to depend on it, the
  document shape, body fields, language and layout, errors, the async API
  (with a file round-trip over `tokio::fs`), and protobuf messages; the
  development notes moved to the end. Its examples are checked against the
  crate.
- When a required body section and front-matter keys are both missing,
  the error is the first missing field in declaration order:
  `ErrorKind::Type` for a front-matter key. Before, the missing body
  section (`ErrorKind::Body`) was reported first.
- `markdown/options.proto`, the `(markdown.body)` option, moved into the crate:
  `rust/markdown/proto/markdown/options.proto`. The import path is the same,
  `markdown/options.proto`. The Cargo dependency now brings the option too, so
  a consumer needs no second channel for it.
- `PROTO_INCLUDE` points into the crate (`<crate>/proto`), so it works for a
  crate fetched alone, not only from a checkout of this repository.

### Removed

- The Bazel label `@serde_markdown//proto/markdown:options.proto`. A Bazel
  consumer drops its `bazel_dep` and `git_override` of `serde_markdown` and
  reaches the file through its crate_universe hub with an annotation
  (`additive_build_file_content` plus `extra_aliased_targets`); the README
  shows it. With one dependency the proto and the crate can no longer drift
  to different versions.
- The fixtures crate `serde_markdown-proto` (not published) no longer carries
  the generated `markdown.body` extension constant.

### Fixed

- Enum fields read back. A field of an enum type (Serde's default,
  externally tagged form) failed to read with `ErrorKind::Type`, in the
  fields block and in a body section. Unit variants and variants with
  content now round-trip in YAML, JSON, and TOML; a unit variant in a body
  section is its raw name.
- `#[derive(Markdown)]` follows the struct's `#[serde(rename_all = "...")]`
  (and `rename_all(serialize = ..., deserialize = ...)`) and drops the `r#`
  of a raw identifier. Before, a body field of such a struct was written
  into the fields block, with no error. A body field whose serialize and
  deserialize names differ is now a compile error.
- A `Vec<u8>` body whose text reads as a YAML or JSON list (`[1, 2]`,
  `- a`) reads back as its bytes; before, it read as the list's numbers or
  failed. A `Vec<u8>` whose text reads as an empty list (`[]`) is
  `ErrorKind::Body` on write, since it cannot be told from an empty list.
- A body field with `#[serde(default)]` may be missing from the document:
  a field without a section is no longer handed to the value, so Serde
  applies its default (or `None`). A required one is still
  `ErrorKind::Body`.
- A TOML date-time in the fields block (`when = 1979-05-27T07:32:00Z`)
  reads into a string or `Timestamp` field; before, `ErrorKind::Type`.
- Built without the `yaml` feature, a document whose first slice is prose
  reads as body, and a body-only document is written with no empty fields
  block. Before, reading failed with `ErrorKind::FormatDisabled` and writing
  added an empty JSON or TOML block.
- Leading `---` lines are skipped in linear time. A document of many of
  them took quadratic time (40 000 lines: 1.4 s).
- Every feature set builds and tests, none at all included.

## [0.3.1] - 2026-10-03

### Fixed

- `FieldsLayout::Bare` wrote a present but empty single body field
  (`Some("")`) without the `---` that ends the fields, so it read back as
  `None`. The separator is now written whenever a body field is present.

## [0.3.0] - 2026-10-03

### Changed

- The last body field is the rest of the document, verbatim: it is never
  parsed or trimmed, so it may hold top-level `---` lines, setext underlines,
  tables, and fences. Before, such a body was written and then refused on read
  (`found 2 sections`) or cut.
- Body text is verbatim in both directions: no line break is added at the end
  of the document or removed from a section. Documents written by 0.3.0 end
  with the last byte of the last section.
- The presence of the last body field is its separator: no separator is
  `None`, a separator followed by nothing is `Some("")`, and `""` after it is
  literal text. The `""` convention for `Some("")` remains only for optional
  body fields that are not the last.
- Front-matter keys are written in declaration order (proto field order for
  buffa messages) in YAML, JSON, and TOML, not alphabetically. No dependency
  was added.
- A bare first slice that is not a mapping is body from its first byte; the
  leading whitespace and `---` prefix is discarded only before fields.

### Added

- A body value that is not the last and holds a top-level `---` line, an
  unclosed fence, or a trailing carriage return is `ErrorKind::Body` at write
  time, instead of a document that cannot be read back.

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
- [0.5.0]
- [0.4.1]
- [0.4.0]
- [0.3.1]
- [0.3.0]
- [0.2.3]
- [0.2.2]
- [0.2.1]
- [0.2.0]
- [0.1.1]
- [0.1.0]

[Unreleased]: https://github.com/sonalect/serde_markdown/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/sonalect/serde_markdown/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/sonalect/serde_markdown/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/sonalect/serde_markdown/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/sonalect/serde_markdown/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/sonalect/serde_markdown/compare/v0.2.3...v0.3.0
[0.2.3]: https://github.com/sonalect/serde_markdown/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/sonalect/serde_markdown/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/sonalect/serde_markdown/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/sonalect/serde_markdown/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/sonalect/serde_markdown/releases/tag/v0.1.1
[0.1.0]: https://github.com/sonalect/serde_markdown/releases/tag/v0.1.0
