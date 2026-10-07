//! Print a sample Markdown page written on a tokio runtime, save it to a
//! file and load it back, and render several pages in parallel tasks.

use example_tokio::{Page, load, render_all, sample, save};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let runtime = tokio::runtime::Builder::new_multi_thread().build()?;
    runtime.block_on(async {
        let page = sample();
        let md = serde_markdown::tokio::to_string(&page).await?;
        println!("{md}");

        let path = std::env::temp_dir().join(format!("example-tokio-{}.md", std::process::id()));
        save(&path, &page).await?;
        let back = load(&path).await;
        tokio::fs::remove_file(&path).await?;
        if back? != page {
            return Err("file round-trip mismatch".into());
        }

        let pages: Vec<Page> = (1..=3)
            .map(|n| Page {
                title: format!("Page {n}"),
                ..page.clone()
            })
            .collect();
        let documents = render_all(pages).await?;
        eprintln!(
            "round-trip ok; rendered {} pages in parallel",
            documents.len()
        );
        Ok(())
    })
}
