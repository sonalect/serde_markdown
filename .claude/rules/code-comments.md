---
paths:
  - "rust/**/*.rs"
---

# Code comments are the API docs

Rustdoc is generated from comments. A reader of those pages does not have
the design documents open. A pointer to a document is not an explanation.

## Coverage

Every **public** (`pub`) item on a handwritten crate surface needs rustdoc
(`///` or `//!`): crate, module, struct, enum, variant, union, trait,
associated type/const, `fn`, method, `const`, `static`, type alias, and
`pub` field.

`pub(crate)` and private items do not. Generated sources
(`rust/proto/markdown/**`, `*_pb.rs`, bindgen) do not: they may stay under
`#![allow(warnings)]` at the file root. A comment in a `.proto` becomes the
rustdoc of what is generated from it, so a proto item is documented in the
proto.

When you add or change a public item, write the rustdoc in the same edit.
When you touch a public item that has none, add it. Do not rephrase an
existing comment unless you are already editing that item.

```rust
// BAD — public method, no rustdoc
pub fn kind(&self) -> ErrorKind {
    self.kind
}

// GOOD
/// The class of the failure, which a caller branches on.
pub fn kind(&self) -> ErrorKind {
    self.kind
}
```

## What the comment says

Do **not** cite `DESIGN.md`, `ROADMAP.md`, their section numbers, or stage
codes (`M7`) in code comments (`//`, `///`, `//!`). Do not markdown-link
them.

Put the **sense** in the comment: what the item does, the invariant, the
failure, the default. Take that text from the docs if needed; do not leave
the citation.

```rust
// BAD
/// Sniff unlabeled fences (DESIGN.md §2.6, stage M7).

// GOOD
/// Shape-based guess for an unlabeled fence or a bare first slice.
/// `{` / `[` means JSON; a TOML-shaped first line means TOML; else YAML.
```

Allowed in comments: public names, on-disk paths, error kinds and `code`
strings, proto field names, fence languages (`yaml`, `json`, `toml`). Those
are the product, not a document index.
