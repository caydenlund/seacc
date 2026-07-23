use crate::{FileId, LineMap};
use token::Token;

mod error;
pub use error::{LexError, LexErrorKind};

pub mod token;

pub fn lex(file: FileId, input: &str) -> (Result<Vec<Token>, Vec<LexError>>, LineMap) {
    todo!()
}
