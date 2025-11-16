//! lex/error: Definition of the [`LexError`] type for errors during lexing

/// Errors that can occur during lexical analysis
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexError {
    /// An unexpected character was encountered
    UnexpectedCharacter {
        /// The unexpected character
        character: char,
        /// Line number where error occurred (1-indexed)
        line: usize,
        /// Column number where error occurred (1-indexed)
        column: usize,
    },
    /// An unterminated string literal
    UnterminatedString {
        /// Line number where string started (1-indexed)
        line: usize,
        /// Column number where string started (1-indexed)
        column: usize,
    },
    /// An invalid numeric literal
    InvalidNumber {
        /// The invalid number text
        text: String,
        /// Line number where error occurred (1-indexed)
        line: usize,
        /// Column number where error occurred (1-indexed)
        column: usize,
    },
}
