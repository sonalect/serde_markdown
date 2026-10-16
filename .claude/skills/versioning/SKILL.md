---
name: versioning
description: >-
  Bump the crates and the Bazel module of this repository in lockstep (Rust-style
  0.x or SemVer after 1.0.0), update CHANGELOG.md, create the matching GitHub tag,
  and publish the GitHub Release. Use when releasing, tagging, publishing a
  GitHub release, bumping version, editing MODULE.bazel or workspace package
  version, or adding a CHANGELOG section.
---

# Versioning

One release version for the whole repository: the crates (`serde_markdown`
and every workspace member) and the Bazel module `serde_markdown`. Document
versions in `DESIGN.md` / `ROADMAP.md` (`Version 0.1`) are **not** this
number.

## Scheme

Versions are `MAJOR.MINOR.PATCH`. `module(version = "0.1.0")` and Cargo
`version = "0.1.0"` have **no** `v`. The git tag **does**: `v0.1.0`.

If **major is 0** (Rust approach, also stated in `CHANGELOG.md`):

- Breaking change → bump **minor** (`0.1.0` → `0.2.0`)
- Bugfix or compatible feature → bump **patch** (`0.1.0` → `0.1.1`)

If **major > 0** (standard SemVer):

- Breaking → **major**
- Compatible features → **minor**
- Bugfixes → **patch**

Do not skip numbers. Do not jump to `1.0.0` unless the user explicitly wants a
stable public API.

**Breaking** means consumer-visible (`.claude/rules/consumers.md`): a
renamed or removed public item (function, type, trait, feature), a changed
error kind, a changed `(markdown.body)` option (file, place, import path,
field number), a removed Bazel label a consumer uses, or a document an
earlier release wrote that this one cannot read. Toolchain pins
(Bazel, rustc) are a patch (0.x) or minor (≥1.0) unless they force a
breaking change.

Pinned `bazel_utils_*` / `rules_rust` / Buf versions are **not** this
number.

## Dependency moves

Renovate's merges leave `CHANGELOG.md` alone
(`.claude/rules/changelog-before-commit.md`). Before moving
`## [Unreleased]` into the release section, add what they changed for
consumers:

```bash
git diff "vPREVIOUS" HEAD -- Cargo.toml rust/markdown/Cargo.toml rust/markdown_derive/Cargo.toml
```

The dependencies are caret ranges, and a consumer resolves its own
versions, so what moved for it is a raised lower bound: every range whose
floor moved, for a crate `serde_markdown` or `serde_markdown_derive` lists
under `[dependencies]` (an optional one included), goes into one
`### Changed` line: `` Dependencies: at least `serde` 1.0.230, `toml`
1.1.7. `` A move of `Cargo.lock` alone changes nothing for a consumer and
stays out. Crates only the examples, tests, or
`serde_markdown-proto` use, and tooling (Bazel, `bazel_utils`, buf, the
protoc plugins, the Rust toolchain, GitHub Actions), stay out. A major
move of a crate whose types the public API exposes (`buffa`, `serde`) is
breaking: bump as for a breaking change.

## Lockstep files

Set the same `MAJOR.MINOR.PATCH` everywhere:

- Root `MODULE.bazel`: `module(version = …)`
- Root `Cargo.toml`: `[workspace.package] version` (members use
  `version.workspace = true`)
- `CHANGELOG.md`: move `## [Unreleased]` notes into `## [MAJOR.MINOR.PATCH] -
  YYYY-MM-DD`; add the version to `## Links` and the reference definitions
  (`[Unreleased]` compare from the new tag, `[MAJOR.MINOR.PATCH]` release URL)

- Root `Cargo.toml`: the exact version of `serde_markdown_derive` in
  `[workspace.dependencies]` (`version = "=MAJOR.MINOR.PATCH"`): the
  published library requires its own derive.
- `README.md`: the `serde_markdown = "…"` examples name the compatible
  range, `MAJOR.MINOR` while major is 0 (`"0.5"`), `MAJOR` after; change
  them only when a release leaves that range.

Refresh `Cargo.lock`, `cargo-bazel-lock.json`
(`.claude/rules/cargo-bazel-lock.md`), and `MODULE.bazel.lock` if they
change.

## Release

After the version commit is on the default branch, create an annotated tag,
push it, and **publish a GitHub Release for that tag** (only when the user
asked to release). A tag without a published Release is incomplete. Do not
leave the Release as a draft. Do not mark it prerelease unless the user
asked. Do not backfill older tags.

```bash
git tag -a "vMAJOR.MINOR.PATCH" -m "vMAJOR.MINOR.PATCH"
git push origin "vMAJOR.MINOR.PATCH"
```

Release notes are the Keep a Changelog body for `## [MAJOR.MINOR.PATCH]` in
`CHANGELOG.md` (after that heading, until the next second-level heading,
so `## Links` stays out). Fail if that
heading is missing. Do not use `--generate-notes` or `--notes-from-tag`.

```bash
notes="$(mktemp)"
# The skill loader substitutes positional placeholders, so this block
# avoids them; escape each `.` of the version in the pattern.
sed -n '/^## \[MAJOR\.MINOR\.PATCH\]/,/^## /{/^## /d;p;}' CHANGELOG.md \
  | sed '/./,$!d' > "$notes"
test -s "$notes"

gh release create "vMAJOR.MINOR.PATCH" \
  --title "vMAJOR.MINOR.PATCH" \
  --notes-file "$notes" \
  --verify-tag
rm -f "$notes"
```

Run `git` / `gh` from the clone root with unrestricted permissions. Return
the Release URL when it succeeds.

## Publish on crates.io

When the GitHub Release is published, the owner publishes both crates from
the tag in one call. Cargo orders them (the derive first, the library
requires it) and, in the dry run, checks the library against the local
derive, which a separate dry run of the library could not find on
crates.io:

```bash
git checkout vMAJOR.MINOR.PATCH
cargo publish --locked --dry-run -p serde_markdown_derive -p serde_markdown
cargo publish --locked -p serde_markdown_derive -p serde_markdown
git checkout main
```

It needs the owner's crates.io token (`cargo login`); do not run the
publish itself. A published version can only be yanked, never replaced,
so a release with a problem gets a new patch version. State and next
steps: `.claude/notes/crates-io-publish.md`.

Examples: `v0.1.0`, `v0.1.1`, `v0.2.0`.
