# serde_markdown

Serde format crate: serialize and deserialize Rust structs (and buffa protobuf
messages) as Markdown documents. Mapping: [DESIGN.md](DESIGN.md). Stages:
[ROADMAP.md](ROADMAP.md).

## Crate

Default serialize is a labeled `yaml` fence (`Format::Yaml` +
`FieldsLayout::Fenced`). JSON and TOML only via `to_string_with_format` or
`to_string_with`. `from_str` / `from_slice` / `from_reader` take
`DeserializeOwned`. Match `Error::kind()`; kinds are `Syntax`, `FrontMatter`,
`Body`, `Type`, `FormatDisabled`, `Io`.

```rust
use serde::{Deserialize, Serialize};
use serde_markdown::{from_str, to_string, Markdown};

#[derive(Debug, PartialEq, Serialize, Deserialize, Markdown)]
struct Page {
    title: String,
    #[markdown(body)]
    body: String,
}

let page = Page {
    title: "Hi".into(),
    body: "Hello".into(),
};
let md = to_string(&page)?;
let back: Page = from_str(&md)?;
```

Also `to_vec` / `to_writer`. Feature `buffa` (off-default) adds
`annotate_markdown_body` and `PROTO_INCLUDE`.

| Feature | Default | Role |
| --- | --- | --- |
| `yaml` | yes | YAML fence dump/load (`yaml_serde`) |
| `json` | yes | JSON fence dump/load (`serde_json` is always the fields IR) |
| `toml` | yes | TOML fence dump/load |
| `derive` | yes | `#[derive(Markdown)]` |
| `buffa` | no | `annotate_markdown_body` |

## Layout

| Path | Role |
| --- | --- |
| [`DESIGN.md`](DESIGN.md) | Format, mapping, proto options, Bazel |
| [`ROADMAP.md`](ROADMAP.md) | Work order: parse, ser/de, derive, buffa (M0–M14) |
| [`CHANGELOG.md`](CHANGELOG.md) | Notable changes |
| [`proto/markdown`](proto/markdown) | Public `(markdown.body)` option |
| [`proto/markdown/testdata`](proto/markdown/testdata) | Fixture messages (not public API) |
| [`rust/serde_markdown/testdata`](rust/serde_markdown/testdata) | Golden Markdown documents |
| [`rust/generated`](rust/generated) | Buffa stubs from `proto/markdown` |
| [`rust/serde_markdown`](rust/serde_markdown) | Format crate |
| [`rust/serde_markdown_derive`](rust/serde_markdown_derive) | `#[derive(Markdown)]` |
| [`MODULE.bazel`](MODULE.bazel) | Bazel module: Rust, Buf, linters |

## Proto generate

Writes `rust/generated/markdown/` (`bazel test //proto/markdown:generate_test`
checks they match):

```bash
bazel run //proto/markdown:generate
```

`google/protobuf/*.proto` comes from `buf.build/protocolbuffers/wellknowntypes`.
If the Buf extension reports `imported file does not exist`, run `buf dep update`
(refreshes [`buf.lock`](buf.lock)) and reload the window.

## Lint and format

`bazel test` never rewrites files; format is `bazel run` only.

```bash
bazel test //proto/markdown:lint   # Protobuf (buf)
bazel test //rust:lint             # clippy
bazel test //bazel:lint            # Starlark (buildifier)
bazel test //bazel:markdown        # markdownlint-cli2
bazel test //rust:vuln             # cargo-audit (needs network)

bazel run //proto/markdown:format  # buf format
bazel run //bazel:format           # buildifier -mode=fix
```

`cargo test` / `cargo clippy` still work for local iteration.

See [CHANGELOG.md](CHANGELOG.md).
