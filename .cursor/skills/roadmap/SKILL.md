---
name: roadmap
description: >-
  Update ROADMAP.md before implementing a new feature or making any
  non-bugfix change; skip for bug fixes that restore specified behavior.
  Use when starting a feature, amending DESIGN/ROADMAP stages, changing
  parse/ser/de/API/proto, or when the user says implement, do M-stage,
  or add a capability.
---

# Roadmap first

Update `ROADMAP.md` **before** implementing a new feature or making any
changes. For bug fixes it is not necessary.

## When to update (before code)

- New feature, public API, parse/split, ser/de, proto option, fence
  language, error kind, or on-disk Markdown shape.
- Work that is not already an unchecked box in the current stage.
- An existing ROADMAP stage the user asked to start: set **Status:**
  `in progress` on that stage (and the dashboard) before writing `src/`.

If DESIGN.md has not decided the path, amend DESIGN.md first. Do not
invent a silent default in `src/`. Do not reopen DESIGN.md §12 as a new
stage; implement those rows in M boxes.

## When to skip

- **Bug fixes** that restore behavior already specified in DESIGN.md /
  existing tests / a done stage.
- rustfmt, comment-only, Clippy, `.cursor/`, changelog-only, or CI pin
  edits that do not change the contract.
- The ROADMAP/DESIGN edit itself.

## How to edit ROADMAP.md

Keep the existing stage format (title, **Status:**, **Codes:**, boxes,
**Review:**). English.

1. Prefer amending the next not-done stage over inventing a parallel
   track. Do not interleave a later M stage with an earlier one.
2. If the work is new and does not fit a current box, add boxes (or a
   new stage after the last planned M) **unchecked**. Dashboard row
   matches.
3. Set **Status:** `in progress` before implementation. Do not set
   `done` until every box is `[x]`, Clippy is clean, `bazel test //...`
   has exited 0, and the owner has reviewed.
4. Tick `[x]` only when the cited check exists in tests (or the owner
   accepted a spec diff). Do not tick from a verbal claim.
5. Stop after the stage. Do not start the next M from a claim that the
   previous one is ready.

```text
# BAD — implement JSON bare serialize, ROADMAP still "not started" / no box
# GOOD — add or tick-in-progress the M7 box, then write src/

# BAD — hotfix wrong sniff for `{` and also rewrite ROADMAP M2 as undone
# GOOD — bug fix only; leave ROADMAP
```
