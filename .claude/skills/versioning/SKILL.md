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

## Lockstep files

Set the same `MAJOR.MINOR.PATCH` everywhere:

- Root `MODULE.bazel`: `module(version = …)`
- Root `Cargo.toml`: `[workspace.package] version` (members use
  `version.workspace = true`)
- `CHANGELOG.md`: move `## [Unreleased]` notes into `## [MAJOR.MINOR.PATCH] -
  YYYY-MM-DD`; add the version to `## Links` and the reference definitions
  (`[Unreleased]` compare from the new tag, `[MAJOR.MINOR.PATCH]` release URL)

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

Examples: `v0.1.0`, `v0.1.1`, `v0.2.0`.
