use crate::{
    Span, Spanned,
    lex::token::{Punct, Token, TokenKind},
    parse::ast::{BinaryOp, Expr, ExprId, UnaryOp},
};

use super::Parser;

/*
# C operator precedence:

1. Left-to-right
    `++`            Suffix/postfix increment
    `--`            Suffix/postfix decrement
    `()`            Function call
    `[]`            Array subscripting
    `.`             Structure and union member access
    `->`            Structure and union member access through pointer
    `(type){list}`  Compound literal
2. Right-to-left
    `++`      Prefix increment
    `--`      Prefix decrement
    `+`       Unary plus
    `-`       Unary minus
    `!`       Logical NOT
    `~`       Bitwise NOT
    `(type)`  Cast
    `*`       Indirection (dereference)
    `&`       Address-of
    `sizeof`  Size-of
3. Left-to-right
    `*`  `%`  `/`
4. Left-to-right
    `+`  `-`
5. Left-to-right
    `<<`  `>>`
6. Left-to-right
    `<`  `<=`  `>`  `>=`
7. Left-to-right
    `==`  `!=`
8. Left-to-right
    `&`  Bitwise AND
9. Left-to-right
    `^`  Bitwise XOR (exclusive or)
10. Left-to-right
    `|`  Bitwise OR (inclusive or)
11. Left-to-right
    `&&`  Logical AND
12. Left-to-right
    `||`  Logical OR
13. Right-to-left
    `?:`  Ternary conditional
14. Right-to-left
    `=`  `+=`  `-=`  `*=`  `/=`  `%=`  `<<=`  `>>=`  `&=`  `^=`  `|=`
15. Left-to-right
    `,`  Comma
*/

impl TokenKind {
    #[inline]
    const fn prefix_bp(&self) -> Option<(UnaryOp, u8)> {
        match self {
            Self::Punct(Punct::Bang) => Some((UnaryOp::Not, 28)),
            Self::Punct(Punct::Minus) => Some((UnaryOp::Negate, 28)),
            _ => None,
        }
    }

    #[inline]
    fn infix_bp(&self) -> Option<(BinaryOp, u8, u8)> {
        let left = |op, bp| Some((op, bp, bp + 1));
        let right = |op, bp| Some((op, bp, bp));
        match self {
            Self::Punct(Punct::Star) => left(BinaryOp::Mul, 26),
            Self::Punct(Punct::Slash) => left(BinaryOp::Div, 26),
            Self::Punct(Punct::Percent) => left(BinaryOp::Mod, 26),
            Self::Punct(Punct::Plus) => left(BinaryOp::Add, 24),
            Self::Punct(Punct::Minus) => left(BinaryOp::Sub, 24),
            Self::Punct(Punct::Lshift) => left(BinaryOp::Lshift, 22),
            Self::Punct(Punct::Rshift) => left(BinaryOp::Rshift, 22),
            Self::Punct(Punct::Lt) => left(BinaryOp::Lt, 20),
            Self::Punct(Punct::LtEq) => left(BinaryOp::LtEq, 20),
            Self::Punct(Punct::Gt) => left(BinaryOp::Gt, 20),
            Self::Punct(Punct::GtEq) => left(BinaryOp::GtEq, 20),
            Self::Punct(Punct::EqEq) => left(BinaryOp::Eq, 18),
            Self::Punct(Punct::BangEq) => left(BinaryOp::Neq, 18),
            Self::Punct(Punct::And) => left(BinaryOp::LogicAnd, 10),
            Self::Punct(Punct::Or) => left(BinaryOp::LogicOr, 8),
            Self::Punct(Punct::Eq) => right(BinaryOp::Assign, 4),
            Self::Punct(Punct::StarEq) => right(BinaryOp::MulAssign, 4),
            Self::Punct(Punct::SlashEq) => right(BinaryOp::DivAssign, 4),
            Self::Punct(Punct::PercentEq) => right(BinaryOp::ModAssign, 4),
            Self::Punct(Punct::PlusEq) => right(BinaryOp::AddAssign, 4),
            Self::Punct(Punct::MinusEq) => right(BinaryOp::SubAssign, 4),
            Self::Punct(Punct::LshiftEq) => right(BinaryOp::LshiftAssign, 4),
            Self::Punct(Punct::RshiftEq) => right(BinaryOp::RshiftAssign, 4),
            _ => None,
        }
    }
}

impl Parser<'_> {
    #[inline]
    pub(super) fn parse_expr(&mut self) -> ExprId {
        self.parse_expr_bp(0)
    }

    fn parse_expr_bp(&mut self, min_bp: u8) -> ExprId {
        let mut lhs = self.parse_expr_prefix();

        loop {
            if let Some(expr) = self.parse_expr_postfix(lhs, min_bp) {
                lhs = expr;
                continue;
            }

            let Some((op, lbp, rbp)) = self.peek().value.infix_bp() else {
                break;
            };

            if lbp < min_bp {
                break;
            }

            self.next();
            let rhs = self.parse_expr_bp(rbp);
            lhs = self.push_expr(
                Expr::Binary { lhs, op, rhs },
                self.ast.exprs[lhs.0 as usize].span + self.ast.exprs[rhs.0 as usize].span,
            );
        }

        lhs
    }

    fn parse_expr_prefix(&mut self) -> ExprId {
        let Token {
            value: tok,
            span: start,
        } = self.next().clone();
        if let Some((op, bp)) = tok.prefix_bp() {
            let operand = self.parse_expr_bp(bp);
            return self.push_expr(
                Expr::Unary { op, operand },
                start + self.ast.expr(operand).span,
            );
        }

        match tok {
            TokenKind::Ident(s) => self.push_expr(Expr::Ident(s), start), // TODO: handle types
            TokenKind::Integer(n) => self.push_expr(Expr::Integer(n), start),
            TokenKind::Decimal(f) => self.push_expr(Expr::Decimal(f), start),
            TokenKind::String(s) => self.push_expr(Expr::String(s), start),
            TokenKind::Punct(Punct::Lparen) => self.parse_expr_parens(start),
            _ => todo!("invalid token for expression: {tok:?}"),
        }
    }

    fn parse_expr_postfix(&mut self, lhs: ExprId, min_bp: u8) -> Option<ExprId> {
        const POSTFIX_BP: u8 = 30;

        if POSTFIX_BP < min_bp {
            return None;
        }

        match self.peek().value {
            TokenKind::Punct(Punct::Lsquare) => {
                let start = self.next().span;
                let ind = self.parse_expr_bp(0);
                let Token {
                    value: tok,
                    span: end,
                } = self.next();
                let end = *end;
                assert!(
                    matches!(tok, &TokenKind::Punct(Punct::Rsquare)),
                    "expr: expected Rsquare but found {tok:?}"
                );
                Some(self.push_expr(Expr::GetIndex { obj: lhs, ind }, start + end))
            }
            _ => None,
        }
    }

    fn parse_expr_parens(&mut self, start: Span) -> ExprId {
        let e = self.parse_expr_bp(0);
        let tok = &self.next().value;
        assert!(
            matches!(tok, &TokenKind::Punct(Punct::Rparen)),
            "expr: expected Rparen but found {tok:?}"
        );
        e
    }

    fn push_expr(&mut self, expr: Expr, span: Span) -> ExprId {
        self.ast.exprs.push(Spanned { value: expr, span });
        #[allow(clippy::cast_possible_truncation)]
        ExprId((self.ast.exprs.len() - 1) as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_binop() {
        let tokens = crate::lex::lex(7, "1 * 2\n").0.expect("should tokenize");
        let mut parser = Parser::new(&tokens);
        assert_eq!(parser.parse_expr(), ExprId(2));
        assert_eq!(parser.ast.expr(ExprId(0)).value, Expr::Integer(1));
        assert_eq!(parser.ast.expr(ExprId(1)).value, Expr::Integer(2));
        assert_eq!(
            parser.ast.expr(ExprId(2)).value,
            Expr::Binary {
                lhs: ExprId(0),
                op: BinaryOp::Mul,
                rhs: ExprId(1)
            }
        );

        let tokens = crate::lex::lex(7, "1 / 2\n").0.expect("should tokenize");
        let mut parser = Parser::new(&tokens);
        assert_eq!(parser.parse_expr(), ExprId(2));
        assert_eq!(parser.ast.expr(ExprId(0)).value, Expr::Integer(1));
        assert_eq!(parser.ast.expr(ExprId(1)).value, Expr::Integer(2));
        assert_eq!(
            parser.ast.expr(ExprId(2)).value,
            Expr::Binary {
                lhs: ExprId(0),
                op: BinaryOp::Div,
                rhs: ExprId(1)
            }
        );

        let tokens = crate::lex::lex(7, "1 % 2\n").0.expect("should tokenize");
        let mut parser = Parser::new(&tokens);
        assert_eq!(parser.parse_expr(), ExprId(2));
        assert_eq!(parser.ast.expr(ExprId(0)).value, Expr::Integer(1));
        assert_eq!(parser.ast.expr(ExprId(1)).value, Expr::Integer(2));
        assert_eq!(
            parser.ast.expr(ExprId(2)).value,
            Expr::Binary {
                lhs: ExprId(0),
                op: BinaryOp::Mod,
                rhs: ExprId(1)
            }
        );

        let tokens = crate::lex::lex(7, "1 + 2\n").0.expect("should tokenize");
        let mut parser = Parser::new(&tokens);
        assert_eq!(parser.parse_expr(), ExprId(2));
        assert_eq!(parser.ast.expr(ExprId(0)).value, Expr::Integer(1));
        assert_eq!(parser.ast.expr(ExprId(1)).value, Expr::Integer(2));
        assert_eq!(
            parser.ast.expr(ExprId(2)).value,
            Expr::Binary {
                lhs: ExprId(0),
                op: BinaryOp::Add,
                rhs: ExprId(1)
            }
        );

        let tokens = crate::lex::lex(7, "1 - 2\n").0.expect("should tokenize");
        let mut parser = Parser::new(&tokens);
        assert_eq!(parser.parse_expr(), ExprId(2));
        assert_eq!(parser.ast.expr(ExprId(0)).value, Expr::Integer(1));
        assert_eq!(parser.ast.expr(ExprId(1)).value, Expr::Integer(2));
        assert_eq!(
            parser.ast.expr(ExprId(2)).value,
            Expr::Binary {
                lhs: ExprId(0),
                op: BinaryOp::Sub,
                rhs: ExprId(1)
            }
        );
    }

    #[test]
    fn binop_prec() {
        let tokens = crate::lex::lex(7, "1 + 2 * 3 - 4\n")
            .0
            .expect("should tokenize");
        let mut parser = Parser::new(&tokens);
        assert_eq!(parser.parse_expr(), ExprId(6));
        assert_eq!(parser.ast.expr(ExprId(0)).value, Expr::Integer(1));
        assert_eq!(parser.ast.expr(ExprId(1)).value, Expr::Integer(2));
        assert_eq!(parser.ast.expr(ExprId(2)).value, Expr::Integer(3));
        assert_eq!(
            parser.ast.expr(ExprId(3)).value,
            Expr::Binary {
                lhs: ExprId(1),
                op: BinaryOp::Mul,
                rhs: ExprId(2)
            }
        );
        assert_eq!(
            parser.ast.expr(ExprId(4)).value,
            Expr::Binary {
                lhs: ExprId(0),
                op: BinaryOp::Add,
                rhs: ExprId(3)
            }
        );
        assert_eq!(parser.ast.expr(ExprId(5)).value, Expr::Integer(4));
        assert_eq!(
            parser.ast.expr(ExprId(6)).value,
            Expr::Binary {
                lhs: ExprId(4),
                op: BinaryOp::Sub,
                rhs: ExprId(5)
            }
        );
    }
}
