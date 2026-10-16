# Publishing to crates.io

Published on 16 October 2026 by the owner, by hand with a crates.io token
(`cargo publish -p serde_markdown_derive -p serde_markdown` from the tag
`v0.5.1`): [`serde_markdown`](https://crates.io/crates/serde_markdown) and
[`serde_markdown_derive`](https://crates.io/crates/serde_markdown_derive)
0.5.1, owned by the owner's GitHub account. Every release publishes both
by hand: "Publish on crates.io" in `.claude/skills/versioning/SKILL.md`.

## What the publish needed (in the tree since 0.5.1)

- Both names were free on crates.io (`serde_markdown`,
  `serde_markdown_derive`; also `serde_markdown-proto` and
  `serde-markdown`). `serde_markdown-proto` and the examples stay
  `publish = false`.
- Caret ranges in `[workspace.dependencies]` instead of `=` pins; the
  quarantine lives in `Cargo.lock` (`.claude/rules/dependency-quarantine.md`)
  and Renovate moves the lock within the ranges (`update-lockfile`).
- `serde_markdown_derive` in `[workspace.dependencies]` has
  `version = "=…"` next to its `path`: `cargo publish` refuses a path-only
  dependency, and the library requires the derive of its own release.
- `rust-version` 1.88, the oldest rustc that builds and tests the published
  crates (1.85 fails on `let` chains; no dependency needs more than 1.85);
  the `msrv` job of `ci.yml` checks it.
- README: absolute GitHub URLs (crates.io breaks relative ones), the
  crates.io dependency (`serde_markdown = "0.5"`), badges. Both packages
  take it as their readme.
- `keywords`, `categories`, `exclude = ["BUILD.bazel"]`, and a copy of
  `LICENSE` in both packages; `[package.metadata.docs.rs] all-features` for
  `serde_markdown`. `proto/` (the `(markdown.body)` option behind
  `PROTO_INCLUDE` and the Bazel annotation) and `testdata/` stay in the
  package.
- `.bazelignore` with `target`: `cargo package` unpacks each crate into
  `target/package/` with its `BUILD.bazel`, which `//...` would pick up.
- The publish step in `.claude/skills/versioning/SKILL.md`; the derive's
  exact version is a lockstep file.

## Still to do

1. **Knowqore** moves from the git tag to `serde_markdown = "0.5"`
   (`.claude/rules/consumers.md`).
2. **Trusted Publishing**, so the CI publishes instead of the owner:
   configure it in each crate's crates.io settings (repository
   `sonalect/serde_markdown`), then a publish job on `v*` tags with
   `id-token: write` and the crates.io auth action pinned by commit SHA.
3. The README's shields.io badges cache a crate's answer for one and four
   hours: a badge loaded before the publish shows "not found" until then.

**Why:** the next publish starts from this state.
