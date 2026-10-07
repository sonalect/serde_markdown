# serde_markdown

A Serde format for Markdown documents with a structured fields block and
ordered body sections. A struct's ordinary fields go into a YAML, JSON, or TOML
block at the top of the document; the fields you mark as body become Markdown
sections after it. It works for hand-written structs and for
[buffa](https://github.com/anthropics/buffa) protobuf messages, and it has a
native async form for tokio.

The complete mapping is in [DESIGN.md](DESIGN.md); changes are in
[CHANGELOG.md](CHANGELOG.md).

## Use

Depend on a release tag:

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_markdown = { git = "https://github.com/sonalect/serde_markdown", tag = "vX.Y.Z" }
```

Replace `vX.Y.Z` with a [release](https://github.com/sonalect/serde_markdown/releases).
The optional parts are [features](#features).

## Quick start

```rust
use serde::{Deserialize, Serialize};
use serde_markdown::{Markdown, from_str, to_string};

#[derive(Debug, PartialEq, Serialize, Deserialize, Markdown)]
struct Note {
    title: String,
    tags: Vec<String>,
    #[markdown(body)]
    summary: String,
    #[markdown(body)]
    text: String,
}

fn main() -> Result<(), serde_markdown::Error> {
    let note = Note {
        title: "Release notes".into(),
        tags: vec!["markdown".into(), "serde".into()],
        summary: "What changed, in one line.".into(),
        text: "The details.\n\n---\n\nThe last body field keeps every byte, `---` lines included.".into(),
    };
    let md = to_string(&note)?;
    let back: Note = from_str(&md)?;
    assert_eq!(back, note);
    Ok(())
}
```

`to_string` writes:

````markdown
```yaml
title: Release notes
tags:
- markdown
- serde
```
What changed, in one line.
---
The details.

---

The last body field keeps every byte, `---` lines included.
````

## The document

- **Fields block.** The fields not marked as body, in declaration order, in a
  fenced `yaml`, `json`, or `toml` code block, or the same mapping with no
  fence, ended by a `---` line. A struct with only body fields writes none.
- **Body sections.** One per `#[markdown(body)]` field, in declaration order,
  separated by `---` lines. Field names do not appear in the document.
- **The last body field** is the rest of the document, verbatim: it may hold
  `---` lines, code fences, tables, anything. An earlier body field ends at the
  next top-level `---` line (outside any list, quote, or code block); `***`
  and `___` do not split.
- **Text is verbatim** both ways: no byte is added to a section or removed
  from it, and the document ends with the last byte of the last section.

Reading finds the fields block by itself: the first fenced `yaml` / `yml` /
`json` / `toml` block, or a mapping before the first top-level `---` line.
An unlabeled fence or a bare block is sniffed: `{` or `[` is JSON, a
`key = value` line is TOML, anything else YAML. A later fence is ordinary body
text, and blank lines or `---` lines before the fields are skipped.

## Body fields

`#[derive(Markdown)]` lists the fields marked `#[markdown(body)]` by their
Serde names, so a field's `rename` and the struct's `rename_all` apply. The
other Serde attributes keep working (`default`, `skip_serializing_if`, …).
Without the derive, implement the trait by hand:

```rust
impl serde_markdown::Markdown for Note {
    const BODY_FIELDS: &'static [&'static str] = &["summary", "text"];
}
```

A body field may have any type. What its section holds follows from its
`Serialize`:

| Value | Section |
| --- | --- |
| `String`, or a type serialized as a string (`Timestamp`, `Duration`, …) | the text itself |
| `Vec<u8>` | its bytes, as UTF-8 text |
| a unit enum variant | the variant's name |
| numbers, structs, maps, lists, variants with content | encoded in the fields block's language (YAML when there is none) |

Body fields may be absent (`Option`, `#[serde(default)]`). An absent field in
the middle leaves an empty section; absent fields at the end leave out their
separators. In a section that is not the last, `Some("")` is written as the two
characters `""`. A document with too few sections gives `None` or the default;
a missing required section is `ErrorKind::Body`. DESIGN.md §4.4 has every case.

## Language and layout

`to_string` writes a fenced YAML block. Choose another language or the bare
layout when writing; reading needs no hint:

```rust
use serde_markdown::{FieldsLayout, Format, to_string_with, to_string_with_format};

let json = to_string_with_format(&note, Format::Json)?; // a ```json fence
let toml = to_string_with(&note, Format::Toml, FieldsLayout::Bare)?;
```

The bare layout has no fence; a `---` line ends the fields:

```markdown
title = "Release notes"
tags = [
    "markdown",
    "serde",
]
---
What changed, in one line.
---
The details.

---

The last body field keeps every byte, `---` lines included.
```

`to_vec` and `from_slice` work on bytes, `to_writer` and `from_reader` on
`std::io::Write` and `std::io::Read`. Reading needs `T: DeserializeOwned`.

## Errors

Every function fails with `serde_markdown::Error`. Branch on `error.kind()`,
not on its text:

| `ErrorKind` | When |
| --- | --- |
| `Syntax` | a code fence before the last body section is not closed, or the bytes are not UTF-8; `error.offset()` is the byte offset |
| `FrontMatter` | the fields block does not decode in its language; the decoder's error is the `source()` |
| `Body` | a section does not fit its body field: a required one is missing or a structured one does not decode; on write, a value that is not the last would split on read |
| `Type` | the value does not fit the document: a missing or mistyped key, or a root that is not a named struct |
| `FormatDisabled` | the document's language is not compiled in |
| `Io` | `to_writer` / `from_reader` I/O; the `std::io::Error` is the `source()` |

## Async API (tokio)

Feature `tokio` adds the module `serde_markdown::tokio`: the async twin of
every function, under the same name, with the same arguments, the same
document, and the same errors.

```toml
serde_markdown = { git = "https://github.com/sonalect/serde_markdown", tag = "vX.Y.Z", features = ["tokio"] }
```

| Sync, `serde_markdown::` | Async, `serde_markdown::tokio::` |
| --- | --- |
| `to_string`, `to_string_with_format`, `to_string_with`, `to_vec` | the same names, `.await`ed |
| `from_str`, `from_slice` | the same names, `.await`ed |
| `to_writer(impl std::io::Write, &T)` | `to_writer(impl tokio::io::AsyncWrite + Unpin, &T)` |
| `from_reader(impl std::io::Read)` | `from_reader(impl tokio::io::AsyncRead + Unpin)` |

Switching is a path and an `.await`:

```rust
let md = serde_markdown::tokio::to_string(&note).await?;
let back: Note = serde_markdown::tokio::from_str(&md).await?;
```

Files go through tokio's I/O:

```rust
use std::path::Path;

use tokio::fs::File;
use tokio::io::AsyncWriteExt;

async fn save(path: &Path, note: &Note) -> Result<(), serde_markdown::Error> {
    let mut file = File::create(path).await?;
    serde_markdown::tokio::to_writer(&mut file, note).await?;
    file.flush().await?; // to_writer does not flush, like its sync twin
    Ok(())
}

async fn load(path: &Path) -> Result<Note, serde_markdown::Error> {
    let file = File::open(path).await?;
    serde_markdown::tokio::from_reader(file).await
}
```

How the async form runs:

- **Natively.** It runs the same steps as the sync form and calls
  `tokio::task::consume_budget` between them, which gives the task back to the
  scheduler only when its cooperative budget is spent. A short document never
  yields; a long one shares the thread with other tasks.
- **On your task.** No thread is spawned and nothing runs on the blocking
  pool, so await these functions as they are; do not wrap them in
  `spawn_blocking`. Any runtime flavor works, and outside a runtime nothing
  yields.
- **In bounded steps.** Writing: serialize the value's body fields, encode the
  fields block, check each body section (one step each), assemble the
  document. Reading: split the document, decode the fields block, deserialize
  the value. A value's own `Serialize` or `Deserialize`, a YAML, JSON, or TOML
  codec call, and the CommonMark pass are one step each.
- **`Send`.** A future borrows the value it writes, so it is `Send` when the
  value is `Sync`, and can run in `tokio::spawn` or a `JoinSet`.

The library turns on the tokio features it needs (`rt`, `io-util`); the
runtime is yours. [`rust/examples/tokio`](rust/examples/tokio) is a runnable
program: a file round-trip over `tokio::fs` and pages written in parallel
tasks.

## Protobuf messages (buffa)

Mark body fields in the `.proto` with the option `(markdown.body)`; the file
`markdown/options.proto` ships inside the crate:

```protobuf
syntax = "proto3";

package example;

import "markdown/options.proto";

message Page {
  string title = 1;
  string text = 2 [(markdown.body) = true];
}
```

With feature `buffa`, `serde_markdown::buffa::annotate_markdown_body` reads
the option from a compiled `FileDescriptorSet` and returns the attributes that
make buffa's generated messages `#[derive(Markdown)]` with their body fields.
Pass them to `buffa_build` in a build script, with
`serde_markdown::PROTO_INCLUDE` as an include path and `generate_json(true)`
(body field names are the proto3 JSON names). The generated messages then go
through `to_string` and `from_str` like any struct.
[`rust/examples/protobuf/build.rs`](rust/examples/protobuf/build.rs) is a
complete build script.

### The option in Bazel

The option comes with the Cargo dependency; there is no Bazel module to depend
on as well. crate_universe fetches the crate with its `proto/` directory. Name
the file with an annotation, alias it into the hub, and stage it for buf:

```starlark
# rust.MODULE.bazel, next to the `from_cargo` (or `from_specs`) of hub `crates`
crate.annotation(
    crate = "serde_markdown",
    additive_build_file_content = """
filegroup(
    name = "proto",
    srcs = ["proto/markdown/options.proto"],
    visibility = ["//visibility:public"],
)
""",
    extra_aliased_targets = {"serde_markdown_proto": "proto"},
)
```

```starlark
# BUILD.bazel
load("@bazel_utils_buf//:buf.bzl", "buf_deps")

buf_deps(
    name = "markdown",
    srcs = ["@crates//:serde_markdown_proto"],
    strip_import_prefix = "/proto",  # stages markdown/options.proto
    visibility = ["//visibility:public"],
)
```

Repin the crate_universe lockfile after adding the annotation.

## Features

| Feature | Default | What it adds |
| --- | --- | --- |
| `yaml` | yes | YAML fields blocks (`yaml_serde`) |
| `json` | yes | JSON fields blocks (`serde_json`) |
| `toml` | yes | TOML fields blocks (`toml`) |
| `derive` | yes | `#[derive(Markdown)]` |
| `buffa` | no | `serde_markdown::buffa::annotate_markdown_body` |
| `tokio` | no | `serde_markdown::tokio`, the async API |

A document in a language left out is `ErrorKind::FormatDisabled`. Without
`yaml`, a block that looks like a YAML mapping is that error, and prose is
read as the body.

## Examples

Three runnable programs in [`rust/examples`](rust/examples):

- `example-page`: a hand-written struct, written and read back;
- `example-protobuf`: a protobuf message compiled by its build script, the
  same document;
- `example-tokio`: the async API on a tokio runtime.

```bash
cargo run -p example-page
cargo run -p example-protobuf   # needs protoc (`PROTOC` or PATH)
cargo run -p example-tokio

bazel run //rust/examples/page
bazel run //rust/examples/protobuf
bazel run //rust/examples/tokio
```

## Development

| Path | What |
| --- | --- |
| [`rust/markdown`](rust/markdown) | the crate `serde_markdown` |
| [`rust/markdown/proto`](rust/markdown/proto) | the `(markdown.body)` option, shipped inside the crate |
| [`rust/markdown/testdata`](rust/markdown/testdata) | golden Markdown documents |
| [`rust/markdown_derive`](rust/markdown_derive) | `#[derive(Markdown)]` |
| [`proto/markdown/testdata`](proto/markdown/testdata) | fixture messages of the tests (not public API) |
| [`rust/proto`](rust/proto) | their generated buffa code |
| [`rust/examples`](rust/examples) | the runnable examples |
| [`DESIGN.md`](DESIGN.md) | the mapping, the API, Bazel and Buf |
| [`ROADMAP.md`](ROADMAP.md) | work stages M0–M16 |

The fast loop, and the gate before a commit (tests, clippy, cargo audit,
buf lint, the generate check, buildifier, markdownlint):

```bash
cargo test --workspace --all-features   # --all-features builds the async form too
cargo clippy --workspace --all-targets --all-features

bazel test //...
```

After changing a fixture `.proto`, regenerate `rust/proto/markdown` (the gate
fails while it differs):

```bash
bazel run //proto/markdown:generate
```

Formatting is `bazel run` only; `bazel test` never rewrites files:

```bash
bazel run //proto/markdown:format  # buf format
bazel run //bazel:format           # buildifier
```

Buf ships the well-known types, so [`buf.yaml`](buf.yaml) has no dependencies.
It lists `rust/markdown/proto`, `proto/`, and `rust/examples/protobuf/proto`,
so the Buf language server resolves `import "markdown/options.proto"`;
[`buf.gen.rust.yaml`](buf.gen.rust.yaml) generates `proto/` only.
`example-protobuf` compiles its proto in `build.rs` with `protoc`; Bazel takes
the prebuilt protoc of the protobuf toolchain.
