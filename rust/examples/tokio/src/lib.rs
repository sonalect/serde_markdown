//! A hand-written `Page` written and read through the async form of
//! serde_markdown, `serde_markdown::tokio` (feature `tokio`).
//!
//! Every function there is the async twin of the sync function with the
//! same name and writes the same document. It runs the same steps and
//! yields to the scheduler between them, so it is awaited as it is: no
//! `spawn_blocking` around it.

use std::error::Error as StdError;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_markdown::{Error, Markdown};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::task::JoinSet;

/// Sample document: labeled `yaml` fence and two body sections.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Markdown)]
pub struct Page {
    pub title: String,
    pub tags: Vec<String>,
    #[markdown(body)]
    pub summary: String,
    #[markdown(body)]
    pub text: String,
}

/// Canonical fenced YAML document for [`sample`].
pub const EXPECTED_DOCUMENT: &str = "\
```yaml
title: Async pages
tags:
- tokio
- markdown
```
Written and read on a tokio runtime.
---
The async form runs the same steps as the sync one
and yields between them when the task's budget is spent.";

/// Page used by the binary and tests.
#[must_use]
pub fn sample() -> Page {
    Page {
        title: "Async pages".into(),
        tags: vec!["tokio".into(), "markdown".into()],
        summary: "Written and read on a tokio runtime.".into(),
        text: "The async form runs the same steps as the sync one\n\
               and yields between them when the task's budget is spent."
            .into(),
    }
}

/// Write `page` to the file at `path`, awaiting the file I/O.
///
/// `to_writer` does not flush, like its sync twin, so the file is flushed
/// here before it is dropped.
pub async fn save(path: &Path, page: &Page) -> Result<(), Error> {
    let mut file = File::create(path).await?;
    serde_markdown::tokio::to_writer(&mut file, page).await?;
    file.flush().await?;
    Ok(())
}

/// Read a `Page` back from the file at `path`.
pub async fn load(path: &Path) -> Result<Page, Error> {
    let file = File::open(path).await?;
    serde_markdown::tokio::from_reader(file).await
}

/// Write every page as its own task, in parallel on the runtime's worker
/// threads, and return the documents in the order of `pages`.
///
/// A task may move between threads at each yield point, which works because
/// the future of `to_string` is `Send` for a `Sync` value. A task that
/// panicked or was aborted is an error too.
pub async fn render_all(pages: Vec<Page>) -> Result<Vec<String>, Box<dyn StdError + Send + Sync>> {
    let mut tasks = JoinSet::new();
    for (index, page) in pages.into_iter().enumerate() {
        tasks.spawn(async move {
            serde_markdown::tokio::to_string(&page)
                .await
                .map(|md| (index, md))
        });
    }
    let mut documents = Vec::with_capacity(tasks.len());
    while let Some(joined) = tasks.join_next().await {
        documents.push(joined??);
    }
    documents.sort_by_key(|(index, _)| *index);
    Ok(documents.into_iter().map(|(_, md)| md).collect())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{EXPECTED_DOCUMENT, Page, load, render_all, sample, save};

    fn runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .build()
            .expect("runtime")
    }

    /// A file in the test's scratch directory (Bazel's `TEST_TMPDIR`, else
    /// the system's).
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::var_os("TEST_TMPDIR").map_or_else(std::env::temp_dir, PathBuf::from);
        dir.join(format!("{name}-{}.md", std::process::id()))
    }

    #[test]
    fn to_string_matches_expected_document() {
        let md = runtime()
            .block_on(serde_markdown::tokio::to_string(&sample()))
            .expect("serialize");
        assert_eq!(md, EXPECTED_DOCUMENT);
        assert_eq!(md, serde_markdown::to_string(&sample()).expect("sync"));
    }

    #[test]
    fn from_str_round_trip_equals_sample() {
        let back: Page = runtime()
            .block_on(serde_markdown::tokio::from_str(EXPECTED_DOCUMENT))
            .expect("deserialize");
        assert_eq!(back, sample());
    }

    #[test]
    fn a_page_saved_to_a_file_loads_back() {
        let path = scratch("example-tokio-save");
        let back = runtime().block_on(async {
            save(&path, &sample()).await.expect("save");
            load(&path).await.expect("load")
        });
        let on_disk = std::fs::read_to_string(&path).expect("read");
        std::fs::remove_file(&path).expect("remove");
        assert_eq!(on_disk, EXPECTED_DOCUMENT);
        assert_eq!(back, sample());
    }

    #[test]
    fn pages_render_in_parallel_tasks_in_order() {
        let pages: Vec<Page> = (0..8)
            .map(|n| Page {
                title: format!("Page {n}"),
                ..sample()
            })
            .collect();
        let documents = runtime()
            .block_on(render_all(pages.clone()))
            .expect("render");
        assert_eq!(documents.len(), pages.len());
        for (page, md) in pages.iter().zip(&documents) {
            assert_eq!(*md, serde_markdown::to_string(page).expect("sync"));
        }
    }
}
