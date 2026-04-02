use crate::source_reader::SourceError;
use crate::span::Span;
use crate::token::CharacterConstantError;
use std::fmt;

/// Error type for preprocessing lexer operations
#[derive(Debug)]
pub enum PpLexerError<'src> {
    /// Error from source reader
    Source(SourceError<'src>),
    /// Unterminated block comment at EOF
    UnterminatedBlockComment(Span<'src>),
    /// Unterminated string literal at EOF or newline
    UnterminatedStringLiteral(Span<'src>),
    /// Unterminated character constant at EOF or newline
    UnterminatedCharacterConstant(Span<'src>),
    /// Invalid character constant content
    InvalidCharacterConstant(Span<'src>, CharacterConstantError),
}

impl fmt::Display for PpLexerError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PpLexerError::Source(err) => write!(f, "{err}"),
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
            PpLexerError::UnterminatedCharacterConstant(span) => {
                write!(
                    f,
                    "unterminated character constant starting at {}:{}-{}",
                    span.file, span.start.0, span.end.0
                )
            }
            PpLexerError::InvalidCharacterConstant(span, err) => {
                write!(
                    f,
                    "invalid character constant at {}:{}-{}: {err}",
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
