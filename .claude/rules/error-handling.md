---
paths:
  - "rust/**/*.rs"
---

# Error type

The public `Result` uses the handwritten `Error` of `src/error.rs`, in the
shape of `serde_json::Error`. Not anyhow, not thiserror, not a protobuf
error envelope (`code` / `path` / `repeat` / `stack`): those fit a service
API, not a serde format. `std::error::Error`, `Display`, and
`serde::{ser,de}::Error` are implemented by hand.

Callers match `kind()`; they never parse `Display`.

| Kind | When |
| - | - |
| `Syntax` | Unclosed fence, bad info string, scan failure, invalid UTF-8 on `from_slice`. `offset()` is `Some`. |
| `FrontMatter` | Decoder error on the fields slice. `source()` is the decoder. |
| `Body` | Section count, empty required structured section, invalid UTF-8 on `Vec<u8>` serialize, a body value that cannot be read back, decoder error on a structured body section (`source()` set). |
| `Type` | Root is not a named struct; mapping-layer Serde `custom` / `missing_field`. |
| `FormatDisabled` | Fence language not in compiled features. |
| `Io` | `to_writer` / `from_reader`. `source()` is `std::io::Error`. |

Do not add a kind unless `DESIGN.md` §7 / §12 is amended first. No second
error enum in `src/`. Recoverable failure is `Result` (`rust-no-panic.md`).
