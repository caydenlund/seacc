use crate::{Spanned, lex::token::Token};
use ast::Expr;

pub mod ast;

mod error;
pub use error::{ParseError, ParseErrorKind};

mod parser;

pub fn parse(tokens: &[Token]) -> Result<Vec<Spanned<Expr>>, Vec<ParseError>> {
    let parser = parser::Parser::new(tokens);
    parser.parse()
}
