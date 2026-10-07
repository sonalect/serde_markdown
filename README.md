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
`annotate_markdown_body`. `PROTO_INCLUDE` is the include path of the
`(markdown.body)` option (see [Proto option](#proto-option)).

| Feature | Default | Role |
| --- | --- | --- |
| `yaml` | yes | YAML fence dump/load (`yaml_serde`) |
| `json` | yes | JSON fence dump/load (`serde_json` is always the fields IR) |
| `toml` | yes | TOML fence dump/load |
| `derive` | yes | `#[derive(Markdown)]` |
| `buffa` | no | `annotate_markdown_body` |
| `tokio` | no | `serde_markdown::tokio`: async twins of the functions |

With feature `tokio`, every function has an async twin under the same name in
`serde_markdown::tokio`; `to_writer` / `from_reader` take tokio's `AsyncWrite`
/ `AsyncRead`. The async form is native: it runs the same steps as the sync
form and yields between them when the task's cooperative budget is spent, so
it needs no `spawn_blocking`.

```rust
let md = serde_markdown::tokio::to_string(&page).await?;
let back: Page = serde_markdown::tokio::from_str(&md).await?;
```

## Layout

| Path | Role |
| --- | --- |
| [`DESIGN.md`](DESIGN.md) | Format, mapping, proto options, Bazel |
| [`ROADMAP.md`](ROADMAP.md) | Work order: parse, ser/de, derive, buffa, examples (M0–M15) |
| [`CHANGELOG.md`](CHANGELOG.md) | Notable changes |
| [`rust/markdown`](rust/markdown) | Format crate |
| [`rust/markdown/proto`](rust/markdown/proto) | Public `(markdown.body)` option, shipped inside the crate |
| [`rust/markdown/testdata`](rust/markdown/testdata) | Golden Markdown documents |
| [`rust/markdown_derive`](rust/markdown_derive) | `#[derive(Markdown)]` |
| [`proto/markdown/testdata`](proto/markdown/testdata) | Fixture messages (not public API) |
| [`rust/proto`](rust/proto) | Buffa stubs of the fixtures |
| [`rust/examples`](rust/examples) | Runnable `example-page`, `example-protobuf`, and `example-tokio` (async form) |
| [`MODULE.bazel`](MODULE.bazel) | Bazel module: Rust, Buf, linters |

## Examples

`example-page` and `example-protobuf` print the same fenced YAML document and
check `from_str(to_string(x)) == x`. `example-tokio` uses the async form
(feature `tokio`) on a tokio runtime: it prints a document, saves it to a file
with `to_writer` and loads it back with `from_reader` over `tokio::fs`, and
renders several pages in parallel tasks.

```bash
cargo run -p example-page
cargo run -p example-protobuf   # needs protoc (`PROTOC` or PATH)
cargo run -p example-tokio

bazel run //rust/examples/page
bazel run //rust/examples/protobuf
bazel run //rust/examples/tokio
```

## Proto option

`markdown/options.proto` declares `(markdown.body)`. It ships inside the crate,
so the Cargo dependency that brings the library is the only one a consumer
needs; there is no Bazel module to depend on as well.

A build script passes `serde_markdown::PROTO_INCLUDE` to protoc or buffa as an
include path, so `import "markdown/options.proto"` resolves.

In Bazel, crate_universe fetches the crate with its `proto/` directory. Name the
file with an annotation, alias it into the hub, and stage it for buf:

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

## Proto generate

Writes the fixtures' stubs to `rust/proto/markdown/`
(`bazel test //proto/markdown:generate_test` checks they match):

```bash
bazel run //proto/markdown:generate
```

`cargo run -p example-protobuf` compiles `rust/examples/protobuf/proto` in that
crate's `build.rs` and needs `protoc` on `PATH` or `PROTOC`. Bazel uses the
protobuf proto toolchain prebuilt (not `@protobuf//:protoc`, which compiles
from source).

`google/protobuf/*.proto` comes from `buf.build/protocolbuffers/wellknowntypes`.
The workspace [`buf.yaml`](buf.yaml) lists `rust/markdown/proto`, `proto/`, and
`rust/examples/protobuf/proto`, so Buf LSP can resolve
`import "markdown/options.proto"` in the fixtures and the example.
[`buf.gen.rust.yaml`](buf.gen.rust.yaml) `inputs` is `proto/` only, so neither
the option nor the example `Page` is emitted into `rust/proto`. If the Buf extension still reports
`imported file does not exist` for well-known types, run `buf dep update`
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
