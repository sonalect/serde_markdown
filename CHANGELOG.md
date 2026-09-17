# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
While the major version is 0, compatible additions bump the patch; breaking
API changes bump the minor.

## [Unreleased]

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
- Bazel + Buf pipeline in the scheda shape: `buf_module`, lint, format,
  `buf.build/anthropics/buffa` plus `protoc-gen-buffa-packaging`,
  `bazel run //proto/markdown:generate`. [`buf.lock`](buf.lock) pins
  well-known types so Buf LSP can resolve `google/protobuf/*.proto`.
- Workspace crates `serde_markdown` (stub) and `serde-markdown-generated`.
- Markdown split parser choice: `pulldown-cmark`. Fields may be a first
  fenced block or **bare** YAML/JSON/TOML (no ` ``` `) before the first
  `---`; format is sniffed. Leading whitespace and `---` before fields are
  ignored. A first slice that is not a mapping is body-only.
- Default serialize format YAML (`yaml_serde`) as a **fenced** ` ```yaml `
  block; JSON and TOML via `to_string_with_format`; bare (unfenced) fields
  via `FieldsLayout::Bare`. `Option` / proto presence in body sections from
  v1.

### Changed

- Buf CLI `v1.73.0`; `bazel_utils_*` modules `v0.2.6`.

## Links

- [Unreleased]

[Unreleased]: https://github.com/sonalect/serde_markdown/commits/HEAD
