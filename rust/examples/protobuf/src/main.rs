//! Print a sample Markdown page from a generated protobuf message.

use example_protobuf::sample;
use serde_markdown::{from_str, to_string};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let page = sample();
    let md = to_string(&page)?;
    print!("{md}");
    let back: example_protobuf::Page = from_str(&md)?;
    if back != page {
        return Err("round-trip mismatch".into());
    }
    eprintln!("round-trip ok");
    Ok(())
}
