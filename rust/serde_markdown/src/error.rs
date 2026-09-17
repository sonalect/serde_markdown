use std::error::Error as StdError;
use std::fmt;
use std::io;

use serde::{de, ser};

use crate::Format;

/// Failure while reading or writing a Markdown document.
///
/// Callers match [`kind`](Error::kind). Do not parse [`Display`].
pub struct Error {
    inner: Box<Inner>,
}

struct Inner {
    kind: ErrorKind,
    offset: Option<usize>,
    message: Box<str>,
    source: Option<Box<dyn StdError + Send + Sync + 'static>>,
}

/// Classification for [`Error`]. Match this; do not parse [`Display`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// Unclosed fence, bad info string, scan failure, or invalid UTF-8 on byte input.
    /// [`Error::offset`] is `Some`.
    Syntax,
    /// Decoder error on the fields slice. [`std::error::Error::source`] is the decoder.
    FrontMatter,
    /// Section count, empty required structured section, invalid UTF-8 on `Vec<u8>`
    /// serialize, or a decoder error on a structured body section (`source` set then).
    Body,
    /// Root is not a named struct, or a mapping-layer Serde `custom` / `missing_field`.
    Type,
    /// Fence language is not in the compiled crate features.
    FormatDisabled,
    /// `to_writer` / `from_reader`. `source` is [`std::io::Error`].
    Io,
}

impl Error {
    /// Classification for this failure. Match this; do not parse [`Display`].
    pub fn kind(&self) -> ErrorKind {
        self.inner.kind
    }

    /// Byte offset into the input. [`Some`] only for [`ErrorKind::Syntax`].
    pub fn offset(&self) -> Option<usize> {
        self.inner.offset
    }

    /// Unclosed fence, bad info string, scan failure, or invalid UTF-8 on byte input.
    pub fn syntax(offset: usize, msg: impl fmt::Display) -> Self {
        Self::new(
            ErrorKind::Syntax,
            Some(offset),
            format!("syntax error at byte {offset}: {msg}"),
            None,
        )
    }

    /// Decoder error on the fields slice. [`std::error::Error::source`] is `err`.
    pub fn front_matter<E>(err: E) -> Self
    where
        E: Into<Box<dyn StdError + Send + Sync + 'static>>,
    {
        let source = err.into();
        Self::new(
            ErrorKind::FrontMatter,
            None,
            format!("front matter: {source}"),
            Some(source),
        )
    }

    /// Body-section failure without a wrapped decoder error.
    pub fn body(msg: impl fmt::Display) -> Self {
        Self::new(ErrorKind::Body, None, format!("body: {msg}"), None)
    }

    /// Body-section failure with a wrapped decoder error as [`std::error::Error::source`].
    pub fn body_source<E>(msg: impl fmt::Display, err: E) -> Self
    where
        E: Into<Box<dyn StdError + Send + Sync + 'static>>,
    {
        let source = err.into();
        Self::new(
            ErrorKind::Body,
            None,
            format!("body: {msg}: {source}"),
            Some(source),
        )
    }

    /// Mapping-layer type failure. [`Display`] is exactly `msg` (not capitalized,
    /// no trailing period).
    pub fn type_error(msg: impl fmt::Display) -> Self {
        Self::new(ErrorKind::Type, None, msg, None)
    }

    /// Fence language is not in the compiled crate features.
    pub fn format_disabled(format: Format) -> Self {
        Self::new(
            ErrorKind::FormatDisabled,
            None,
            format!("format not enabled: {format}"),
            None,
        )
    }

    /// I/O failure from `to_writer` / `from_reader`. [`std::error::Error::source`] is `err`.
    pub fn io(err: io::Error) -> Self {
        Self::new(
            ErrorKind::Io,
            None,
            format!("I/O error: {err}"),
            Some(Box::new(err)),
        )
    }

    fn new(
        kind: ErrorKind,
        offset: Option<usize>,
        message: impl fmt::Display,
        source: Option<Box<dyn StdError + Send + Sync + 'static>>,
    ) -> Self {
        Self {
            inner: Box::new(Inner {
                kind,
                offset,
                message: message.to_string().into_boxed_str(),
                source,
            }),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::io(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.inner.message)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Error")
            .field("kind", &self.inner.kind)
            .field("offset", &self.inner.offset)
            .field("message", &self.inner.message)
            .finish()
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self.inner.source.as_deref() {
            Some(err) => Some(err),
            None => None,
        }
    }
}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self::type_error(msg)
    }
}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self::type_error(msg)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error as StdError;
    use std::io;

    use serde::{de, ser};

    use super::{Error, ErrorKind};
    use crate::Format;

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn syntax() {
        let err = Error::syntax(12, "unclosed fence");
        assert_eq!(err.kind(), ErrorKind::Syntax);
        assert_eq!(err.offset(), Some(12));
        assert!(StdError::source(&err).is_none());
        let display = err.to_string();
        assert!(display.contains("12"), "{display}");
        assert!(display.contains("unclosed fence"), "{display}");
    }

    #[test]
    fn front_matter_wraps_decoder() {
        let err = Error::front_matter(io::Error::other("decoder failed"));
        assert_eq!(err.kind(), ErrorKind::FrontMatter);
        assert_eq!(err.offset(), None);
        assert!(StdError::source(&err).is_some());
    }

    #[test]
    fn body_without_source() {
        let err = Error::body("expected 2 sections, found 1");
        assert_eq!(err.kind(), ErrorKind::Body);
        assert!(StdError::source(&err).is_none());
    }

    #[test]
    fn body_source() {
        let err = Error::body_source("note", io::Error::other("decoder failed"));
        assert_eq!(err.kind(), ErrorKind::Body);
        assert!(StdError::source(&err).is_some());
    }

    #[test]
    fn root_not_named_struct_is_type() {
        let typed = Error::type_error("root must be a named struct");
        assert_eq!(typed.kind(), ErrorKind::Type);
        assert_eq!(typed.offset(), None);
        assert!(StdError::source(&typed).is_none());

        let de = <Error as de::Error>::custom("root must be a named struct");
        assert_eq!(de.kind(), ErrorKind::Type);
        assert_eq!(de.offset(), None);
        assert!(StdError::source(&de).is_none());

        let ser = <Error as ser::Error>::custom("root must be a named struct");
        assert_eq!(ser.kind(), ErrorKind::Type);
        assert_eq!(ser.offset(), None);
        assert!(StdError::source(&ser).is_none());
    }

    #[test]
    fn missing_field_is_type() {
        let err = <Error as de::Error>::missing_field("field1");
        assert_eq!(err.kind(), ErrorKind::Type);
        assert!(err.to_string().contains("field1"), "{}", err);
    }

    #[test]
    fn format_disabled_json() {
        let err = Error::format_disabled(Format::Json);
        assert_eq!(err.kind(), ErrorKind::FormatDisabled);
        assert_eq!(err.to_string(), "format not enabled: json");
        assert!(StdError::source(&err).is_none());
    }

    #[test]
    fn io_and_from_io_error() {
        let from = Error::from(io::Error::other("disk full"));
        assert_eq!(from.kind(), ErrorKind::Io);
        assert!(StdError::source(&from).is_some());

        let io = Error::io(io::Error::other("disk full"));
        assert_eq!(io.kind(), ErrorKind::Io);
        assert!(StdError::source(&io).is_some());
    }

    #[test]
    fn error_and_kind_are_send_sync() {
        assert_send_sync::<Error>();
        assert_send_sync::<ErrorKind>();
    }
}
