use crate::span::Span;
use std::fmt;
use std::io::Error as IoError;

/// Error type for source file reading
#[derive(Debug)]
pub enum SourceError<'src> {
    /// I/O error while reading the file
    Io(IoError),
    /// Non-empty file must end with an unescaped newline
    MissingFinalNewline,
    /// Invalid UTF-8 character encountered
    InvalidUtf8(Span<'src>),
}

impl fmt::Display for SourceError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceError::Io(err) => write!(f, "I/O error: {err}"),
            SourceError::MissingFinalNewline => {
                write!(f, "source file must end with a newline character")
            }
            SourceError::InvalidUtf8(span) => {
                write!(
                    f,
                    "invalid UTF-8 character at {}:{}-{}",
                    span.file, span.start.0, span.end.0
                )
            }
        }
    }
}

impl std::error::Error for SourceError<'_> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SourceError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<IoError> for SourceError<'_> {
    fn from(err: IoError) -> Self {
        SourceError::Io(err)
    }
}
