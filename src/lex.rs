use crate::{FileId, LineMap};
use token::Token;

mod error;
pub use error::{LexError, LexErrorKind};

pub mod token;

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
