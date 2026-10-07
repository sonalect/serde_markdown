# Dependency list order

When adding, removing, or bumping a dependency, put it in its sorted place.
Do not append at the bottom.

## Cargo.toml

- `[workspace.members]`: alphabetical by path.
- `[workspace.dependencies]`, `[dependencies]`, `[dev-dependencies]`,
  `[build-dependencies]`, per-target tables: two zones, decided by the
  owner (7 October 2026):
  1. this repository's path crates (`serde_markdown`,
     `serde_markdown-proto`, `serde_markdown_derive`), alphabetical;
  2. third-party crates, alphabetical by package name.

  Optional crates stay in the same sort, not in a block at the end.

```toml
# BAD — the path crate among the third-party ones, buffa dumped at the bottom
pulldown-cmark = { workspace = true }
serde = { workspace = true }
serde_markdown_derive = { workspace = true, optional = true }
yaml_serde = { workspace = true, optional = true }
buffa = { workspace = true, optional = true }

# GOOD
serde_markdown_derive = { workspace = true, optional = true }
buffa = { workspace = true, optional = true }
pulldown-cmark = { workspace = true }
serde = { workspace = true }
yaml_serde = { workspace = true, optional = true }
```

## BUILD.bazel

`deps` / `proc_macro_deps` lists: alphabetical. Workspace `//rust/…` first,
then `@serde_markdown_cargo//:…`. `bazel run //bazel:format` sorts what
buildifier can.

Does not replace `dependency-quarantine.md` or the crate_universe repin
(`cargo-bazel-lock.md`).
