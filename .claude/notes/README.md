# Project notes

Project state, deferred work, measurements, and references that must
outlive one machine (see `.claude/rules/project-notes.md`). Read this index
before planning; open a note when its topic comes up.

| Note | What it holds |
| - | - |
| `dependency-updates.md` | Renovate, CI, and the repin workflow: what is tracked, the trial schedule, what to watch |

## References

- Design and decisions: `DESIGN.md` (§12 resolved decisions); work order:
  `ROADMAP.md`.
- Anchor consumer: Knowqore, `github.com/sonalect/knowqore`, usually
  cloned as `../knowqore` (`.claude/rules/consumers.md`). Up to `v0.3.1` it
  pins this library twice: the crate by Cargo git tag, and the Bazel module
  by `bazel_dep` + `git_override` for `proto/markdown/options.proto` (the
  pins drifted: Cargo `v0.3.1`, `bazel_dep` `0.2.3`). From the release
  that moves the option into the crate, it drops the `bazel_dep` and adds
  the crate_universe annotation of the README, "Proto option".
- Bazel helpers: `github.com/sonalect/bazel_utils` (`../bazel_utils`).
