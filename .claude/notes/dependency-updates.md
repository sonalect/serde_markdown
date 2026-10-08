# Dependency updates

Renovate (the Mend Renovate App) opens the update pull requests; CI tests
them. Set up 8 October 2026.

## Pieces

- `.github/renovate.json5`: what Renovate tracks, the groups, the schedule,
  the 2-day quarantine (`dependency-quarantine.md`).
- `.github/workflows/ci.yml`: `bazel test //...` on every pull request and
  on main.
- `.github/workflows/repin.yml`: on a `renovate/` branch, repins
  `cargo-bazel-lock.json` and `MODULE.bazel.lock` and pushes them with the
  secret `REPIN_TOKEN` (fine-grained token, Contents read and write on this
  repository). Renovate cannot run Bazel, so without it every Cargo or
  Bazel update is red.
- Dependabot alerts are on; Renovate opens security updates from them
  outside the schedule.

## What is tracked

| Group | Files | Source |
| - | - | - |
| one PR per crate | `Cargo.toml`, `Cargo.lock` | crates.io |
| `buffa` | the buffa crates, the two `protoc-gen-buffa*` in `buf.MODULE.bazel` | crates.io, `anthropics/buffa` releases |
| `Bazel tooling` | `bazel_dep` in `MODULE.bazel`, `.bazelversion` | BCR, Bazel releases |
| `bazel_utils` | every `bazel_dep` and `git_override` tag of it | `sonalect/bazel_utils` tags, no quarantine |
| `Rust toolchain` | `rust-toolchain.toml`, `RUST_VERSION` in `rust.MODULE.bazel` | Rust release channel (`rust-version`) |
| `buf` (alone) | `buf.toolchains(version)` | `bufbuild/buf` releases |
| `GitHub Actions` | `.github/workflows/*.yml`, pinned by digest | GitHub |

Not tracked: `rust-version` in `Cargo.toml` (the MSRV is a promise to
consumers, moved by hand); transitive crates (no lock-file maintenance:
`cargo update` would ignore the quarantine; `cargo audit` and the alerts
cover vulnerabilities).

**Why buf and buffa fail first:** `bazel_utils` fetches the buf CLI and
the protoc plugins from a catalog of versions with their sha256. A version
missing from it fails the build. Add it to `bazel_utils`, release, and let
the `bazel_utils` update land before the buf or buffa one.

## Changelog

Renovate's pull requests do not touch `CHANGELOG.md`. The dependency
moves consumers see are written at release, from the diff since the last
tag (`versioning` skill, "Dependency moves"); decided 8 October 2026, so
merges stay one click and the PRs never conflict over `## [Unreleased]`.

## Schedule

Trial: every day before 06:00 UTC (`schedule` in `renovate.json5`), so the
flow can be watched. **How to apply:** once the owner trusts it, change it
to weekly, e.g. `['* 0-5 * * 1']`.

## To watch during the trial

- The Dependency Dashboard issue: an update shown as pending forever means
  its datasource gives no release date and the quarantine never lets it
  through (possible for BCR modules). Then decide per group.
- A red PR: the catalog (above), a breaking major, or a repin failure.
