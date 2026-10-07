# Full review

When the owner asks for a **full review**, review **all code and all
docs** in the repository. That means every handwritten source file
(`src/`, `tests/`, the derive macro, BUILD, proto, generated code, golden
documents, examples, scripts), and every document (`DESIGN.md`,
`ROADMAP.md`, README, CHANGELOG, and equivalent). Not the documents alone.
Not the current diff. Not the last stage. Not the focused file.

If the workspace has more than one git root, review the repository the
owner named (or the one the current work is in). Do not fold a sibling
repo (Knowqore, for one) into the same plan unless asked.

Do **not** implement. The deliverable is a **coding plan**. Do the review
**in this session** yourself; do not hand it to a subagent or a weaker
model. A full review runs on Opus: if this session is on another model,
say so and ask the owner to switch (`/model opus`) before starting.

## Checklist

1. **Bugs** — wrong results, broken invariants, bad error kinds or codes, edge cases (empty, unclosed fence, `---` in a body, CRLF, invalid input).
2. **Allocations** — `clone` / `to_owned` / `format!` / extra `Vec` on hot paths; borrow or reuse when it stays clear.
3. **Dead code** — unused, duplicated, leftover `todo!` for work already shipped.
4. **Performance** — extra parse of the same buffer, repeated sniff, N+1.
5. **Docs vs code** — `DESIGN.md`, `ROADMAP.md`, and README match behaviour; no contradiction with `src/` (names, fence languages, codes, defaults).
6. **Spec / API** — public names and error kinds match `DESIGN.md`; no silent defaults the spec left open.
7. **Tests** — stages marked done in `ROADMAP.md` have a real test; golden documents cover what `DESIGN.md` claims; missing negatives.
8. **Errors** — `Result` in `src/` (no `panic!` / `unwrap` / `expect` / `assert!`); the kind or code not replaced by a message.
9. **Untrusted input** — Markdown, YAML/JSON/TOML fields and fences: no panic, no unbounded work on junk input.
10. **Consumers** — everything a consumer needs at build time sits inside the crate (`component.md`); each feature set builds and tests.

Skip an item only when that surface is absent; say so. Do not pad the plan
with cosmetic nits.

## Plan shape

Prioritised list. Each item: severity, location, what's wrong, proposed
change. No patches until the owner asks to implement.
