# Reference data lives in files, not in code

Reference data is any list or table of facts about the world that the code
looks things up in: vocabularies, role sets, name or path patterns, phrase
lists, model families, secret patterns, word lists of a guard test. It
grows, and when something goes wrong it is the first thing to inspect. So
it lives in its own readable file, never in a Rust literal.

This applies everywhere: every crate, tests included, new code and old.

## Where and how

- One file per table, next to the crate that owns it:
  `<crate>/data/<name>.yaml`. YAML, so a human can open it and read it.
- The file starts with a comment: what the table is, who reads it, and the
  rule for adding an entry.
- The code embeds it at compile time (`include_str!`) and parses it into a
  typed struct (`yaml_serde`, `deny_unknown_fields`). Parsing returns
  `Result`; no `panic!`, `unwrap`, or `expect` (`rust-no-panic.md`).
- A unit test parses every file and checks its invariants (no duplicates,
  values from the allowed set), so a broken file fails the gates, not a
  run.
- Bazel: the file goes into the target's `compile_data`.

## Not reference data

- One named constant: a wire name, a rules version, a default.
- Names of our own format: the keys of a file this component itself writes, the
  fields of a proto message, the variants of our own enum.
- Test inputs and expected outputs of one test.

```rust
// BAD — a table hidden in code
const LOCK_FILES: &[&str] = &["Cargo.lock", "uv.lock", "go.sum"];

// GOOD — rust/markdown/data/lock_files.yaml, embedded and parsed
static LOCK_FILES: &str = include_str!("../data/lock_files.yaml");
```
