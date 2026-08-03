use super::ast::{BinaryOp, Expr, ExprId, UnaryOp};
use super::{ParseError, ParseErrorKind};
use crate::lex::token::{Punct, Token, TokenKind};
use crate::{Span, Spanned};

pub(super) struct Parser<'a> {
    exprs: Vec<Spanned<Expr>>,
    tokens: &'a [Token],
    pos: usize,
    errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
    pub(super) const fn new(tokens: &'a [Token]) -> Self {
        Self {
            exprs: Vec::new(),
            tokens,
            pos: 0,
            errors: Vec::new(),
        }
    }

    pub(super) fn parse(mut self) -> Result<Vec<Spanned<Expr>>, Vec<ParseError>> {
        if self.tokens.is_empty() || matches!(self.peek(), Some(TokenKind::Eof)) {
            return Ok(Vec::new());
        }

        if self.parse_equality().is_err() {
            return Err(self.errors);
        }
        while !matches!(self.peek(), Some(TokenKind::Eof) | None) {
            self.unexpected_current();
            self.pos += 1;
        }
        if self.errors.is_empty() {
            Ok(self.exprs)
        } else {
            Err(self.errors)
        }
    }

    fn parse_equality(&mut self) -> Result<ExprId, ()> {
        self.parse_left_associative(
            Self::parse_comparison,
            &[Punct::EqEq, Punct::BangEq],
            &[BinaryOp::Eq, BinaryOp::Neq],
        )
    }

    fn parse_comparison(&mut self) -> Result<ExprId, ()> {
        self.parse_left_associative(
            Self::parse_shift,
            &[Punct::Gt, Punct::GtEq, Punct::Lt, Punct::LtEq],
            &[BinaryOp::Gt, BinaryOp::GtEq, BinaryOp::Lt, BinaryOp::LtEq],
        )
    }

    fn parse_shift(&mut self) -> Result<ExprId, ()> {
        self.parse_left_associative(
            Self::parse_term,
            &[Punct::Lshift, Punct::Rshift],
            &[BinaryOp::Lshift, BinaryOp::Rshift],
        )
    }

    fn parse_term(&mut self) -> Result<ExprId, ()> {
        self.parse_left_associative(
            Self::parse_factor,
            &[Punct::Plus, Punct::Minus],
            &[BinaryOp::Add, BinaryOp::Sub],
        )
    }

    fn parse_factor(&mut self) -> Result<ExprId, ()> {
        self.parse_left_associative(
            Self::parse_unary,
            &[Punct::Star, Punct::Slash],
            &[BinaryOp::Mul, BinaryOp::Div],
        )
    }

    fn parse_left_associative(
        &mut self,
        operand: fn(&mut Self) -> Result<ExprId, ()>,
        puncts: &[Punct],
        ops: &[BinaryOp],
    ) -> Result<ExprId, ()> {
        let mut lhs = operand(self)?;
        while let Some(index) = puncts.iter().position(|punct| self.peek_punct(*punct)) {
            let operator = self.tokens[self.pos].clone();
            self.pos += 1;
            let error_count = self.errors.len();
            let Ok(rhs) = operand(self) else {
                self.errors.truncate(error_count);
                self.push_error(
                    ParseErrorKind::UnexpectedToken(operator.value),
                    operator.span,
                );
                return Err(());
            };
            lhs = self.binary(lhs, ops[index], rhs);
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<ExprId, ()> {
        let (op, start) = if let Some(start) = self.consume_punct(Punct::Minus) {
            (UnaryOp::Negate, start)
        } else if let Some(start) = self.consume_punct(Punct::Bang) {
            (UnaryOp::Not, start)
        } else {
            return self.parse_atom();
        };

        let rhs = self.parse_unary()?;
        Ok(self.push_expr(Expr::UnaryOp(op, rhs), start + self.exprs[rhs].span))
    }

    fn parse_atom(&mut self) -> Result<ExprId, ()> {
        let Some((kind, span)) = self.next() else {
            self.unexpected_current();
            return Err(());
        };
        match kind {
            TokenKind::Integer(value) => Ok(self.push_expr(Expr::Integer(value), span)),
            TokenKind::Decimal(value) => Ok(self.push_expr(Expr::Decimal(value), span)),
            TokenKind::String(value) => Ok(self.push_expr(Expr::String(value), span)),
            TokenKind::Ident(value) => Ok(self.push_expr(Expr::Ident(value), span)),
            TokenKind::Punct(Punct::Lparen) => {
                let inner = self.parse_equality()?;
                let Some(end) = self.consume_punct(Punct::Rparen) else {
                    self.unexpected_current();
                    return Err(());
                };
                self.exprs[inner].span = span + end;
                Ok(inner)
            }
            _ => {
                self.push_error(ParseErrorKind::UnexpectedToken(kind), span);
                Err(())
            }
        }
    }

    fn binary(&mut self, lhs: ExprId, op: BinaryOp, rhs: ExprId) -> ExprId {
        self.push_expr(
            Expr::BinaryOp(lhs, op, rhs),
            self.exprs[lhs].span + self.exprs[rhs].span,
        )
    }

    fn push_expr(&mut self, value: Expr, span: Span) -> ExprId {
        self.exprs.push(Spanned { value, span });
        self.exprs.len() - 1
    }

    fn unexpected_current(&mut self) {
        if let Some(token) = self.tokens.get(self.pos) {
            self.push_error(
                ParseErrorKind::UnexpectedToken(token.value.clone()),
                token.span,
            );
        } else if let Some(token) = self.tokens.last() {
            let span = Span {
                file: token.span.file,
                start: token.span.end,
                end: token.span.end,
            };
            self.push_error(ParseErrorKind::UnexpectedToken(TokenKind::Eof), span);
        }
    }

    fn push_error(&mut self, value: ParseErrorKind, span: Span) {
        self.errors.push(Spanned { value, span });
    }

    fn consume_punct(&mut self, expected: Punct) -> Option<Span> {
        if self.peek_punct(expected) {
            let span = self.tokens[self.pos].span;
            self.pos += 1;
            Some(span)
        } else {
            None
        }
    }

    fn peek_punct(&self, expected: Punct) -> bool {
        matches!(self.peek(), Some(TokenKind::Punct(punct)) if *punct == expected)
    }

    fn next(&mut self) -> Option<(TokenKind, Span)> {
        let token = self.tokens.get(self.pos)?.clone();
        if !matches!(token.value, TokenKind::Eof) {
            self.pos += 1;
        }
        Some((token.value, token.span))
    }

    fn peek(&self) -> Option<&TokenKind> {
        self.tokens.get(self.pos).map(|token| &token.value)
    }
}
