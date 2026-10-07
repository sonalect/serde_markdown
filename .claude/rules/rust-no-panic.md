---
paths:
  - "rust/**/*.rs"
---

# Rust panic policy

## Library code (`src/`)

Allowed only: `todo!`, `unimplemented!`, `unreachable!`.

Forbidden: `panic!`, `unwrap`, `expect`, `unwrap_err`, `unwrap_unchecked`,
`assert!`, `assert_eq!`, `assert_ne!`. Recoverable failure is `Result`
with the crate's `Error` (`error-handling.md`).

```rust
// BAD
let first = info.split_whitespace().next().unwrap();
panic!("missing language");

// GOOD
fn fields_lang(info: &str) -> Result<Option<Sniff>, Error> {
    match info.split_whitespace().next() {
        None | Some("") => Ok(None),
        Some(first) => parse_lang(first),
    }
}

// GOOD — placeholder until the next stage implements it
todo!("functionality is not implemented")
```

## Tests (`tests/`, `#[cfg(test)]`)

`assert!`, `assert_eq!`, `assert_ne!`, `panic!`, `unwrap`, and `expect` are
allowed.
