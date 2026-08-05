use crate::lex::token::Token;
use crate::parse::ast::Ast;

mod decl;
mod expr;
mod stmt;
mod typ;

pub struct ParsedOutput {
    pub ast: Ast,
}

pub(super) struct Parser<'a> {
    ast: Ast,
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    pub(super) fn new(tokens: &'a [Token]) -> Self {
        Self {
            ast: Ast::default(),
            tokens,
            pos: 0,
        }
    }

    pub(super) fn parse(mut self) -> ParsedOutput {
        while self.pos < self.tokens.len() {
            // self.parse_item();
        }

        ParsedOutput { ast: self.ast }
    }

    fn next(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.pos)?;
        self.pos += 1;
        Some(token)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }
}
