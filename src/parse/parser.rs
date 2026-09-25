use crate::ir::{Constant, Graph, Node, NodeId, NodeKind};
use crate::parse::lex::TokenKind;
use crate::parse::{Lexer, ParseError, ParseResult};

#[derive(Clone)]
pub struct Parser<'s> {
    tokens: Lexer<'s>,
    graph: Graph,
}

impl<'src> Parser<'src> {
    pub fn new(src: &'src str) -> Self {
        Self {
            tokens: Lexer::new(src),
            graph: Graph::default(),
        }
    }

    pub fn parse(mut self) -> ParseResult<Graph> {
        todo!()
    }
}
