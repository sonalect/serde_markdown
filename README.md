# serde_markdown

Serde format crate: serialize and deserialize Rust structs (and buffa protobuf
messages) as Markdown documents. Design: [DESIGN.md](DESIGN.md).

## Layout

| Path | Role |
| --- | --- |
| [`DESIGN.md`](DESIGN.md) | Format, mapping, proto options, Bazel |
| [`CHANGELOG.md`](CHANGELOG.md) | Notable changes |
| [`proto/markdown`](proto/markdown) | Public `(markdown.body)` option |
| [`proto/markdown/testdata`](proto/markdown/testdata) | Fixture messages (not public API) |
| [`rust/serde_markdown/testdata`](rust/serde_markdown/testdata) | Golden Markdown documents |
| [`rust/generated`](rust/generated) | Buffa stubs from `proto/markdown` |
| [`rust/serde_markdown`](rust/serde_markdown) | Format crate |
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
