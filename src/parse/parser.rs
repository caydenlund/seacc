use crate::ir::{Constant, ControlId, Graph, ValueId};
use crate::node;
use crate::parse::lex::TokenKind;
use crate::parse::{Lexer, ParseError, ParseResult};

#[derive(Clone)]
pub struct Parser<'s> {
    tokens: Lexer<'s>,
    graph: Graph,
    start: ControlId,
}

impl<'src> Parser<'src> {
    pub fn new(src: &'src str) -> Self {
        let graph = Graph::default();
        let start = graph.start();
        Self {
            tokens: Lexer::new(src),
            graph,
            start,
        }
    }

    pub fn parse(mut self) -> ParseResult<Graph> {
        let value = self.parse_expr()?;
        let _ret = self.graph.add(node![Return(self.start, value)]);

        if let Some(tok) = self.tokens.next()? {
            return Err(ParseError::ExtraInput(tok));
        }

        Ok(self.graph)
    }

    fn parse_expr(&mut self) -> ParseResult<ValueId> {
        match self.tokens.peek()? {
            Some(&TokenKind::Number(n)) => {
                let _ = self.tokens.next();
                Ok(self.graph.add(node![Constant(Constant::Number(n))]))
            }
            _ => todo!(),
        }
    }
}
