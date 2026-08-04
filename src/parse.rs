use crate::lex::token::Token;

pub mod ast;

mod error;
pub use error::{ParseError, ParseErrorKind};

mod parser;
pub use parser::ParsedOutput;

pub fn parse(tokens: &[Token]) -> ParsedOutput {
    parser::Parser::new(tokens).parse()
}
