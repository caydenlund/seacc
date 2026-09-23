mod lexer;
use lexer::Lexer;

mod parser;
use parser::Parser;

use crate::ir::{NodeArena, NodeId};

#[derive(Debug, Clone)]
pub struct ParseResult {
    pub nodes: NodeArena,
    pub start: NodeId,
}

/// Parses the given text into a string
///
/// # Errors
/// When input is invalid
pub fn parse(s: &str) -> Result<ParseResult, String> {
    Parser::new(s).parse()
}
