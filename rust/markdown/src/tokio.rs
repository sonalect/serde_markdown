//! The async form of the crate, behind the feature `tokio`.
//!
//! Each function here has the name, arguments, and result of its sync twin
//! at the crate root and writes or reads the same document: switch
//! `serde_markdown::to_string(&page)` to
//! `serde_markdown::tokio::to_string(&page).await`. [`to_writer`] and
//! [`from_reader`] take tokio's [`AsyncWrite`] and [`AsyncRead`] and await
//! the I/O.
//!
//! The async form is native, not the sync form moved to another thread. Both
//! forms run one algorithm in steps; the async form calls
//! `tokio::task::consume_budget` between them, which gives the task back to
//! the scheduler only when its cooperative budget is spent. A short
//! document never yields, and outside a tokio runtime nothing does. No
//! thread is spawned and nothing runs on the blocking pool, so a caller
//! awaits these functions as they are, with no `spawn_blocking` around them.
//!
//! Writing steps: serialize the value's body fields; encode the fields
//! block; check each body section, one per step; assemble the document.
//! Reading steps: split the document; decode the fields block; deserialize
//! the value. A value's own `Serialize` or `Deserialize`, a YAML, JSON, or
//! TOML codec call, and the `pulldown-cmark` pass over a document are one
//! step each.
//!
//! A future borrows the value it writes, so it is `Send` when the value is
//! `Sync`.
//!
//! ```
//! # #[cfg(all(feature = "derive", feature = "yaml"))]
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use serde::{Deserialize, Serialize};
//! use serde_markdown::Markdown;
//!
//! #[derive(Debug, PartialEq, Serialize, Deserialize, Markdown)]
//! struct Page {
//!     title: String,
//!     #[markdown(body)]
//!     body: String,
//! }
//!
//! let runtime = tokio::runtime::Builder::new_current_thread().build()?;
//! runtime.block_on(async {
//!     let page = Page { title: "Hi".into(), body: "Hello".into() };
//!     let md = serde_markdown::tokio::to_string(&page).await?;
//!     assert_eq!(md, serde_markdown::to_string(&page)?);
//!     let back: Page = serde_markdown::tokio::from_str(&md).await?;
//!     assert_eq!(back, page);
//!     Ok(())
//! })
//! # }
//! # #[cfg(not(all(feature = "derive", feature = "yaml")))]
//! # fn main() {}
//! ```

use ::tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::de::ReadDoc;
use crate::drive;
use crate::error::Error;
use crate::format::{FieldsLayout, Format};
use crate::markdown::Markdown;
use crate::ser::WriteDoc;

/// Serialize `value` as a fenced YAML Markdown document.
///
/// The async twin of [`crate::to_string`]: the same document.
pub async fn to_string<T: Serialize + Markdown>(value: &T) -> Result<String, Error> {
    to_string_with(value, Format::Yaml, FieldsLayout::Fenced).await
}

/// Serialize `value` as a fenced Markdown document in `format`.
///
/// The async twin of [`crate::to_string_with_format`]: the same document.
pub async fn to_string_with_format<T: Serialize + Markdown>(
    value: &T,
    format: Format,
) -> Result<String, Error> {
    to_string_with(value, format, FieldsLayout::Fenced).await
}

/// Serialize `value` with an explicit fence language and fields layout.
///
/// The async twin of [`crate::to_string_with`]: the same document.
pub async fn to_string_with<T: Serialize + Markdown>(
    value: &T,
    format: Format,
    layout: FieldsLayout,
) -> Result<String, Error> {
    drive::run_async(WriteDoc::new(value, format, layout)).await
}

/// Serialize `value` as UTF-8 bytes of a fenced YAML Markdown document.
///
/// The async twin of [`crate::to_vec`]: the same bytes.
pub async fn to_vec<T: Serialize + Markdown>(value: &T) -> Result<Vec<u8>, Error> {
    to_string(value).await.map(String::into_bytes)
}

/// Write `value` as a fenced YAML Markdown document to `writer`.
///
/// The async twin of [`crate::to_writer`]: the same bytes, written with
/// `write_all`. Like its twin it does not flush; flush or shut down a
/// buffered writer yourself. A write failure is
/// [`crate::ErrorKind::Io`] with `source` set to [`std::io::Error`].
pub async fn to_writer<W, T>(mut writer: W, value: &T) -> Result<(), Error>
where
    W: AsyncWrite + Unpin,
    T: Serialize + Markdown,
{
    let text = to_string(value).await?;
    writer.write_all(text.as_bytes()).await?;
    Ok(())
}

/// Deserialize a Markdown document from `s`.
///
/// The async twin of [`crate::from_str`]: the same mapping and failures.
pub async fn from_str<T>(s: &str) -> Result<T, Error>
where
    T: DeserializeOwned + Markdown,
{
    drive::run_async(ReadDoc::new(s)).await
}

/// Deserialize a Markdown document from UTF-8 `bytes`.
///
/// The async twin of [`crate::from_slice`]: invalid UTF-8 is
/// [`crate::ErrorKind::Syntax`] with [`Error::offset`] at the first invalid
/// byte.
pub async fn from_slice<T>(bytes: &[u8]) -> Result<T, Error>
where
    T: DeserializeOwned + Markdown,
{
    match std::str::from_utf8(bytes) {
        Ok(s) => from_str(s).await,
        Err(err) => Err(Error::syntax(err.valid_up_to(), "invalid UTF-8")),
    }
}

/// Deserialize a Markdown document by reading `reader` to the end.
///
/// The async twin of [`crate::from_reader`]: the read is awaited, then the
/// bytes are read as by [`from_slice`]. A read failure is
/// [`crate::ErrorKind::Io`] with `source` set to [`std::io::Error`].
pub async fn from_reader<R, T>(mut reader: R) -> Result<T, Error>
where
    R: AsyncRead + Unpin,
    T: DeserializeOwned + Markdown,
{
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).await?;
    from_slice(&bytes).await
}

#[cfg(all(test, feature = "yaml", feature = "json", feature = "toml"))]
mod tests {
    use std::future::Future;
    use std::io;
    use std::pin::{Pin, pin};
    use std::task::{Context, Poll, Waker};

    use ::tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

    use super::{
        from_reader, from_slice, from_str, to_string, to_string_with, to_string_with_format,
        to_vec, to_writer,
    };
    use crate::error::ErrorKind;
    use crate::format::{FieldsLayout, Format};
    use crate::testdata::{goldens, types, values};

    fn block_on<F: Future>(future: F) -> F::Output {
        ::tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("runtime")
            .block_on(future)
    }

    fn assert_send<F: Future + Send>(future: F) -> F {
        future
    }

    #[test]
    fn every_writer_gives_the_sync_document() {
        let page = values::page();
        block_on(async {
            assert_eq!(
                to_string(&page).await.expect("async"),
                crate::to_string(&page).expect("sync")
            );
            assert_eq!(
                to_vec(&page).await.expect("async"),
                crate::to_vec(&page).expect("sync")
            );
            for format in [Format::Yaml, Format::Json, Format::Toml] {
                assert_eq!(
                    to_string_with_format(&page, format).await.expect("async"),
                    crate::to_string_with_format(&page, format).expect("sync")
                );
                for layout in [FieldsLayout::Fenced, FieldsLayout::Bare] {
                    assert_eq!(
                        to_string_with(&page, format, layout).await.expect("async"),
                        crate::to_string_with(&page, format, layout).expect("sync"),
                        "{format} {layout:?}"
                    );
                }
            }
            let mut written = Vec::new();
            to_writer(&mut written, &page).await.expect("async");
            assert_eq!(written, crate::to_vec(&page).expect("sync"));
        });
    }

    #[test]
    fn every_reader_gives_the_sync_value() {
        block_on(async {
            for (name, golden) in [
                ("yaml", goldens::PAGE_FENCED_YAML),
                ("json", goldens::PAGE_BARE_JSON),
                ("toml", goldens::PAGE_UNLABELED_TOML),
            ] {
                let sync: types::Page = crate::from_str(golden).expect(name);
                let back: types::Page = from_str(golden).await.expect(name);
                assert_eq!(back, sync, "{name}");
                let back: types::Page = from_slice(golden.as_bytes()).await.expect(name);
                assert_eq!(back, sync, "{name}");
                let back: types::Page = from_reader(golden.as_bytes()).await.expect(name);
                assert_eq!(back, sync, "{name}");
            }
            let article: types::Article = from_str(goldens::STRUCTURED_FENCED_YAML)
                .await
                .expect("structured body");
            assert_eq!(article, values::article());
        });
    }

    #[test]
    fn the_async_form_fails_as_the_sync_form_does() {
        block_on(async {
            let err = from_slice::<types::FieldsOnly>(&[0xff, b'a'])
                .await
                .expect_err("utf8");
            assert_eq!(err.kind(), ErrorKind::Syntax);
            assert_eq!(err.offset(), Some(0));

            let err = from_str::<types::FieldsOnly>("```yaml\nname: only\n")
                .await
                .expect_err("unclosed");
            assert_eq!(err.kind(), ErrorKind::Syntax);

            let mut page = values::page();
            page.text1 = "before\n---\nafter".into();
            let err = to_string(&page).await.expect_err("middle value with ---");
            assert_eq!(err.kind(), ErrorKind::Body);
        });
    }

    struct Broken;

    impl AsyncRead for Broken {
        fn poll_read(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            _buf: &mut ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
            Poll::Ready(Err(io::Error::other("boom")))
        }
    }

    impl AsyncWrite for Broken {
        fn poll_write(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            _buf: &[u8],
        ) -> Poll<io::Result<usize>> {
            Poll::Ready(Err(io::Error::other("boom")))
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    #[test]
    fn an_io_failure_is_io() {
        block_on(async {
            let err = from_reader::<_, types::FieldsOnly>(Broken)
                .await
                .expect_err("read");
            assert_eq!(err.kind(), ErrorKind::Io);
            assert!(std::error::Error::source(&err).is_some());

            let err = to_writer(Broken, &values::page()).await.expect_err("write");
            assert_eq!(err.kind(), ErrorKind::Io);
            assert!(std::error::Error::source(&err).is_some());
        });
    }

    #[test]
    fn no_call_needs_a_thread_or_a_runtime() {
        let page = values::page();
        let mut write = pin!(assert_send(to_string(&page)));
        let Poll::Ready(md) = write.as_mut().poll(&mut Context::from_waker(Waker::noop())) else {
            panic!("outside a runtime the write finishes in one poll");
        };
        let md = md.expect("write");
        let mut read = pin!(assert_send(from_str::<types::Page>(&md)));
        let Poll::Ready(back) = read.as_mut().poll(&mut Context::from_waker(Waker::noop())) else {
            panic!("outside a runtime the read finishes in one poll");
        };
        assert_eq!(back.expect("read"), page);
    }
}
