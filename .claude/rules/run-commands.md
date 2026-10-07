# Run commands from this repository

Every `bazel`, `cargo`, and `buf` command **must** run in this clone's
root: the directory with `MODULE.bazel` and the workspace `Cargo.toml`.
The Bash tool's working directory persists between calls and may have
drifted (another repository may be the session's primary directory), so
`cd` to the root by absolute path in the same command when unsure. Never
run them from a sibling project, a scratch or tmp directory, or a
subdirectory.

## Generate

The generated Rust of `proto/markdown` is checked in under
`rust/proto/markdown`. After changing a `.proto`, regenerate it before
tests with `bazel run //proto/markdown:generate`, and read the diff. The
generated files are test fixtures, not public API.
`//proto/markdown:generate_test` fails while the tree differs from the
plugins' output. Never edit a generated file by hand.

## Fast loop and gate

- `cargo test --workspace` and
  `cargo clippy --workspace --all-targets --all-features`: the fast loop.
  `example-protobuf` compiles its proto in `build.rs` and needs `protoc`
  on `PATH` or in `PROTOC`.
- `bazel test //... --keep_going --test_output=errors`: the gate before a
  commit (`commit-after-tests.md`). It also runs `//bazel:lint`
  (buildifier), `//bazel:markdown` (markdownlint over every `*.md`,
  `.claude/` included), `//proto/markdown:lint` (buf lint),
  `//proto/markdown:generate_test`, `//rust:lint` (clippy), and
  `//rust:vuln` (cargo audit). `bazel run //bazel:format` fixes BUILD
  formatting; `bazel run //proto/markdown:format` fixes proto formatting.

## No sandbox

Do not run `bazel`, `buf`, or `cargo` inside a command sandbox. Bazelisk
needs the network (`.bazelversion`, BCR, crate_universe), the cache lives
under `~/.cache/bazel`, `buf` needs the Buf Schema Registry, and generate
writes into the source tree. If Bash sandboxing is on, run these with
`dangerouslyDisableSandbox: true`.
