# Consumers outside this repository

serde_markdown is a general-purpose library for any Rust project, other
people's included. Knowqore (`github.com/sonalect/knowqore`, usually
cloned next to this repository as `../knowqore`) is its **anchor
consumer**: the first user and the source of requirements for current
work. It is not the only one, and the library is not built for it alone.

## Requirements from the anchor consumer

- Generalize a Knowqore need into a capability any consumer could use.
  The code, names, defaults, and docs never mention Knowqore or its domain
  (`component.md`).
- A Knowqore example may motivate a decision in `DESIGN.md`; the
  normative text states the general rule.
- When a Knowqore need and a clean general design disagree, say so to the
  owner instead of bending the library toward one caller.

## When a change can break a consumer

Consumer-visible:

- the public API: its functions, `Format`, `FieldsLayout`, the `Markdown`
  trait and derive, `annotate_markdown_body`, `PROTO_INCLUDE`, `Error` and
  `ErrorKind`;
- the `(markdown.body)` option: its file and place in the crate
  (`rust/markdown/proto/markdown/options.proto`), import path, package, and
  field number; and the Cargo features;
- the bytes: a document an earlier release wrote that this one reads
  differently or refuses, or different bytes written for the same value.

Before calling such a change done:

1. Write the effect into `CHANGELOG.md` under `Changed` / `Removed`, in
   words any caller can act on.
2. Bump per `.claude/skills/versioning/SKILL.md` at release: breaking →
   minor while major is 0.
3. Check Knowqore as a real-world sample:
   `git grep -n serde_markdown ../knowqore` (or ask `scout`), and tell the
   owner which of its call sites change. Name every pin it has: the Cargo
   tag, and a `bazel_dep` if it still keeps one for `options.proto`
   (`component.md`). Knowqore passing is a sample, not proof that other
   consumers are safe.

Do not edit a consumer's repository from this session unless the owner
asks. `readiness.md` § "A plan that crosses repositories" applies.
