# Changelog before commit

Before creating a git commit, make `CHANGELOG.md` **actual** for that
commit. Do not commit API, proto, on-disk, behaviour, or user-facing docs
changes while `## [Unreleased]` still omits them.

## What to write

Keep a Changelog groups: `Added`, `Changed`, `Fixed`, `Removed`. English.
The user-facing *why* / effect, not a file list. New notes go under
`## [Unreleased]`. Do **not** bump the crate / `MODULE.bazel` version here:
that is a release (`.claude/skills/versioning/SKILL.md`).

If Unreleased already states the change, leave it. If it is stale or
wrong, edit it.

```text
# BAD — the bare layout changed, Unreleased empty
## [Unreleased]

# GOOD
## [Unreleased]
### Fixed
- `FieldsLayout::Bare` writes the `---` that ends the fields before an empty
  body, so `Some("")` no longer reads back as `None`.
```

## Skip (nothing notable)

- The commit is **only** `CHANGELOG.md`.
- rustfmt / comment-only / agent rules (`.claude/**`) with
  no library, proto, or `DESIGN.md` change.
- Clippy / rustc lint-only: same public API, same written bytes, same
  runtime behaviour.
- CI and bot configuration (`.github/**`).
- A dependency update that only moves tooling: Bazel, Bazel modules,
  `bazel_utils`, buf, the protoc plugins, the Rust toolchain, GitHub
  Actions, or a crate only the examples and tests use.
- The owner explicitly waived the changelog for this commit.

A Renovate pull request carries no `CHANGELOG.md` edit, even when it moves
a crate `serde_markdown` or `serde_markdown_derive` depends on (the `=` pin
forces that version on consumers). Those moves are written at release,
from the diff since the last tag (`.claude/skills/versioning/SKILL.md`,
"Dependency moves"). Consumers take tags, not main, so `## [Unreleased]`
only has to be complete when the tag is cut. A security update is
released at once and gets its line in that release.

Do not skip for "small" changes to the public API, error kinds, fence
languages, the `(markdown.body)` option, or how a document is split,
written, or read. A lint fix that changes a failure or
what the caller sees is not lint-only.
