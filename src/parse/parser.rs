use super::ParseErrorKind;
use crate::lex::token::{Punct, Token, TokenKind};
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

    fn next(&mut self) -> &Token {
        let t = &self.tokens[self.pos];
        if self.pos < self.tokens.len() - 1 && !matches!(t.value, TokenKind::Eof) {
            self.pos += 1;
        }
        t
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }
}
