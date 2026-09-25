mod lex;
use lex::Lexer;
pub use lex::{LexError, Token};

mod parser;
use parser::Parser;

use crate::ir::Graph;

#[derive(Debug, Clone)]
pub enum ParseError {
    LexError(LexError),
    ExtraInput(Token),
}

impl From<LexError> for ParseError {
    fn from(err: LexError) -> Self {
        Self::LexError(err)
    }
}

pub type ParseResult<T> = Result<T, ParseError>;

/// Parses the given text into a string
///
/// # Errors
/// When input is invalid
pub fn parse(s: &str) -> ParseResult<Graph> {
    Parser::new(s).parse()
}
