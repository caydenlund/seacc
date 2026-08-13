use crate::{FileId, LineMap};
use token::Token;

pub mod token;

mod error;
pub use error::{LexError, LexErrorKind};

mod lexer;

pub fn lex(file: FileId, input: &str) -> (Result<Vec<Token>, Vec<LexError>>, LineMap) {
    let mut lexer = lexer::Lexer::new(file, input);
    lexer.lex();

    let result = if lexer.errors.is_empty() {
        Ok(lexer.tokens)
    } else {
        Err(lexer.errors)
    };
    (result, LineMap::from(lexer.lines.into_boxed_slice()))
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// Lexes the given string (with an added newline), panicking on error
    ///
    /// # Panics
    /// On error
    #[must_use]
    pub fn lex(input: &str) -> Vec<Token> {
        super::lex(7, &(String::from(input) + "\n"))
            .0
            .unwrap_or_else(|e| panic!("unable to lex input '{input:?}': {e:?}"))
    }
}
