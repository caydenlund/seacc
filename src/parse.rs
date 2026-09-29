mod lex;
use lex::Lexer;
pub use lex::{LexError, Token};

mod parser;
use parser::Parser;

use crate::util::arena::{Arena, ArenaId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    LexError(LexError),
    EmptyInput,
    ExtraInput,
    UnclosedList,
    ExtraRparen,
}

impl From<LexError> for ParseError {
    fn from(err: LexError) -> Self {
        Self::LexError(err)
    }
}

#[derive(Debug, Clone)]
pub enum AstNode {
    Number(i64),
    String(String),
    Symbol(String),
    List(Vec<AstNodeId>),
}

pub type AstNodeId = ArenaId<AstNode>;

#[derive(Debug, Clone)]
pub struct Ast {
    pub nodes: Arena<AstNode>,
    pub root: AstNodeId,
}

pub type ParseResult<T> = Result<T, ParseError>;

/// Parses the given text into an abstract syntax tree
///
/// # Errors
/// Returns a [`ParseError`] when input is invalid
pub fn parse(s: &str) -> ParseResult<Ast> {
    Parser::new(s).parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn serialize(ast: &Ast) -> String {
        fn serialize_rec(ast: &Ast, node: AstNodeId) -> String {
            match ast.nodes.get(node).unwrap() {
                AstNode::Number(n) => n.to_string(),
                AstNode::String(s) => format!("\"{s}\""),
                AstNode::Symbol(s) => s.clone(),
                AstNode::List(inner) => format!(
                    "({})",
                    inner
                        .iter()
                        .map(|n| serialize_rec(ast, *n))
                        .collect::<Vec<_>>()
                        .join(" ")
                ),
            }
        }
        serialize_rec(ast, ast.root)
    }

    fn parse(s: &str) -> ParseResult<String> {
        super::parse(s).map(|ast| serialize(&ast))
    }

    #[test]
    fn atom() {
        assert_eq!(parse("123"), Ok("123".into()));
        assert_eq!(parse("abc"), Ok("abc".into()));
        assert_eq!(parse("\"xyz\""), Ok("\"xyz\"".into()));
    }

    #[test]
    fn list() {
        assert_eq!(parse("(1)"), Ok("(1)".into()));
        assert_eq!(parse("(1 2 3)"), Ok("(1 2 3)".into()));
        assert_eq!(parse("(A (AA () BB) B)"), Ok("(A (AA () BB) B)".into()));
    }

    #[test]
    fn invalid() {
        assert_eq!(parse("(1"), Err(ParseError::UnclosedList));
        assert_eq!(parse("((1)"), Err(ParseError::UnclosedList));
        assert_eq!(parse("(1))"), Err(ParseError::ExtraRparen));
        assert_eq!(parse("1)"), Err(ParseError::ExtraRparen));
        assert_eq!(parse("1 2"), Err(ParseError::ExtraInput));
        assert_eq!(parse(""), Err(ParseError::EmptyInput));
        assert_eq!(
            parse("\""),
            Err(ParseError::LexError(LexError::UnterminatedString))
        );
    }
}
