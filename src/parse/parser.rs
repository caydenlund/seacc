use super::ParseErrorKind;
use crate::lex::token::Token;
use crate::parse::ast::{Ast, BinaryOp, Expr, ExprId, ItemId, UnaryOp};
use crate::{Span, Spanned};

mod expr;

pub struct ParsedOutput {
    ast: Ast,
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
        todo!();

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
