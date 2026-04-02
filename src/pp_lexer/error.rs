use crate::source_reader::SourceError;
use crate::span::{Span, Spanned};
use std::fmt;

/// Error type for preprocessing lexer operations
#[derive(Debug)]
pub enum PpLexerError<'src> {
    /// Error from source reader
    Source(SourceError<'src>),
    /// Unexpected character encountered
    UnexpectedChar(Spanned<'src, char>),
    /// Unterminated block comment at EOF
    UnterminatedBlockComment(Span<'src>),
    /// Unterminated string literal at EOF or newline
    UnterminatedStringLiteral(Span<'src>),
}

impl fmt::Display for PpLexerError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PpLexerError::Source(err) => write!(f, "{err}"),
            PpLexerError::UnexpectedChar(ch) => {
                write!(
                    f,
                    "unexpected character '{}' at {}:{}-{}",
                    ch.value, ch.span.file, ch.span.start.0, ch.span.end.0
                )
            }
            PpLexerError::UnterminatedBlockComment(span) => {
                write!(
                    f,
                    "unterminated block comment starting at {}:{}-{}",
                    span.file, span.start.0, span.end.0
                )
            }
            PpLexerError::UnterminatedStringLiteral(span) => {
                write!(
                    f,
                    "unterminated string literal starting at {}:{}-{}",
                    span.file, span.start.0, span.end.0
                )
            }
        }
    }
}

impl std::error::Error for PpLexerError<'_> {}

impl<'src> From<SourceError<'src>> for PpLexerError<'src> {
    fn from(err: SourceError<'src>) -> Self {
        PpLexerError::Source(err)
    }
}
