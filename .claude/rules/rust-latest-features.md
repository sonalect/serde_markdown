---
paths:
  - "rust/**/*.rs"
---

# Latest Rust features

Workspace floor is `[workspace.package] rust-version` and `edition` in the
root `Cargo.toml` (1.88 since 16 October 2026, the oldest rustc that builds
and tests the published crates), not the newer pinned toolchain. Write for
that compiler: CI builds and tests with it, and clippy's
`incompatible_msrv` flags a newer std API. Do not write as if MSRV were an
older rustc from training data.

When writing or changing Rust, **look up** language, std, and Cargo features
stabilized after your knowledge cutoff and up through the workspace MSRV.
Training data is stale. Do not rely on memory for what is stable.

Sources: [Rust release notes](https://doc.rust-lang.org/stable/releases.html),
[Rust blog](https://blog.rust-lang.org/), and current std docs. Check the last
several stable releases (1.96, 1.97, 1.98, … — whatever is current), not only
the newest one.

Use a newer **stable** feature when it makes this code shorter, clearer, or
more efficient (including replacing a crate or a hand-rolled helper with std).
Skip it when it does not help, changes semantics, or would require an
unrelated mass-refactor. Nightly/`feature` gates are forbidden unless the
crate already uses them.
