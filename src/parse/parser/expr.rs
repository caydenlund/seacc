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
                // `[]`: Array subscripting
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
            TokenKind::Punct(Punct::Lparen) => {
                // `()`: Function call
                let start = self.next().span;
                let mut args = Vec::new();

                if self.peek().value != TokenKind::Punct(Punct::Rparen) {
                    loop {
                        args.push(self.parse_expr_bp(0));
                        match self.peek().value {
                            TokenKind::Punct(Punct::Comma) => {
                                self.next();
                            }
                            TokenKind::Punct(Punct::Rparen) => {
                                break;
                            }
                            _ => {
                                panic!("expected Comma or Rparen but found {:?}", self.peek().value)
                            }
                        }
                    }
                }

                let Token {
                    value: tok,
                    span: end,
                } = self.next();
                let end = *end;
                assert!(
                    matches!(tok, &TokenKind::Punct(Punct::Rparen)),
                    "expr: expected Rparen but found {tok:?}"
                );
                Some(self.push_expr(Expr::Call { callee: lhs, args }, start + end))
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
    use std::cell::RefCell;

    use super::*;

    fn parse_expr(input: &str) -> Vec<Expr> {
        let tokens = crate::lex::lex(7, &format!("{input}\n"))
            .0
            .expect("should tokenize");
        let mut parser = Parser::new(&tokens);
        parser.parse_expr();
        parser.ast.exprs.into_iter().map(|s| s.value).collect()
    }

    #[derive(Default, Clone)]
    struct ExprBuilder {
        exprs: RefCell<Vec<Expr>>,
    }
    impl ExprBuilder {
        fn last_id(&self) -> ExprId {
            #[allow(clippy::cast_possible_truncation)]
            ExprId((self.exprs.borrow().len() - 1) as u32)
        }

        fn ident(&self, s: &str) -> ExprId {
            self.exprs.borrow_mut().push(Expr::Ident(s.into()));
            self.last_id()
        }

        fn int(&self, value: u64) -> ExprId {
            self.exprs.borrow_mut().push(Expr::Integer(value));
            self.last_id()
        }

        fn float(&self, value: f64) -> ExprId {
            self.exprs.borrow_mut().push(Expr::Decimal(value));
            self.last_id()
        }

        fn string(&self, value: &str) -> ExprId {
            self.exprs.borrow_mut().push(Expr::String(value.into()));
            self.last_id()
        }

        fn binop(&self, lhs: ExprId, op: BinaryOp, rhs: ExprId) -> ExprId {
            self.exprs.borrow_mut().push(Expr::Binary { lhs, op, rhs });
            self.last_id()
        }

        fn unary(&self, op: UnaryOp, operand: ExprId) -> ExprId {
            self.exprs.borrow_mut().push(Expr::Unary { op, operand });
            self.last_id()
        }

        fn index(&self, obj: ExprId, ind: ExprId) -> ExprId {
            self.exprs.borrow_mut().push(Expr::GetIndex { obj, ind });
            self.last_id()
        }

        fn mul(&self, lhs: ExprId, rhs: ExprId) -> ExprId {
            self.binop(lhs, BinaryOp::Mul, rhs)
        }

        fn modu(&self, lhs: ExprId, rhs: ExprId) -> ExprId {
            self.binop(lhs, BinaryOp::Mod, rhs)
        }

        fn div(&self, lhs: ExprId, rhs: ExprId) -> ExprId {
            self.binop(lhs, BinaryOp::Div, rhs)
        }

        fn add(&self, lhs: ExprId, rhs: ExprId) -> ExprId {
            self.binop(lhs, BinaryOp::Add, rhs)
        }

        fn sub(&self, lhs: ExprId, rhs: ExprId) -> ExprId {
            self.binop(lhs, BinaryOp::Sub, rhs)
        }

        fn assign(&self, lhs: ExprId, rhs: ExprId) -> ExprId {
            self.binop(lhs, BinaryOp::Assign, rhs)
        }

        fn mul_assign(&self, lhs: ExprId, rhs: ExprId) -> ExprId {
            self.binop(lhs, BinaryOp::MulAssign, rhs)
        }

        fn modu_assign(&self, lhs: ExprId, rhs: ExprId) -> ExprId {
            self.binop(lhs, BinaryOp::ModAssign, rhs)
        }

        fn div_assign(&self, lhs: ExprId, rhs: ExprId) -> ExprId {
            self.binop(lhs, BinaryOp::DivAssign, rhs)
        }

        fn add_assign(&self, lhs: ExprId, rhs: ExprId) -> ExprId {
            self.binop(lhs, BinaryOp::AddAssign, rhs)
        }

        fn sub_assign(&self, lhs: ExprId, rhs: ExprId) -> ExprId {
            self.binop(lhs, BinaryOp::SubAssign, rhs)
        }

        fn lshift(&self, lhs: ExprId, rhs: ExprId) -> ExprId {
            self.binop(lhs, BinaryOp::Lshift, rhs)
        }

        fn rshift(&self, lhs: ExprId, rhs: ExprId) -> ExprId {
            self.binop(lhs, BinaryOp::Rshift, rhs)
        }

        fn eq(&self, lhs: ExprId, rhs: ExprId) -> ExprId {
            self.binop(lhs, BinaryOp::Eq, rhs)
        }

        fn neq(&self, lhs: ExprId, rhs: ExprId) -> ExprId {
            self.binop(lhs, BinaryOp::Neq, rhs)
        }

        fn lt(&self, lhs: ExprId, rhs: ExprId) -> ExprId {
            self.binop(lhs, BinaryOp::Lt, rhs)
        }

        fn lt_eq(&self, lhs: ExprId, rhs: ExprId) -> ExprId {
            self.binop(lhs, BinaryOp::LtEq, rhs)
        }

        fn gt(&self, lhs: ExprId, rhs: ExprId) -> ExprId {
            self.binop(lhs, BinaryOp::Gt, rhs)
        }

        fn gt_eq(&self, lhs: ExprId, rhs: ExprId) -> ExprId {
            self.binop(lhs, BinaryOp::GtEq, rhs)
        }

        fn logic_and(&self, lhs: ExprId, rhs: ExprId) -> ExprId {
            self.binop(lhs, BinaryOp::LogicAnd, rhs)
        }

        fn logic_or(&self, lhs: ExprId, rhs: ExprId) -> ExprId {
            self.binop(lhs, BinaryOp::LogicOr, rhs)
        }

        fn call(&self, callee: ExprId, args: Vec<ExprId>) -> ExprId {
            self.exprs.borrow_mut().push(Expr::Call { callee, args });
            self.last_id()
        }
    }

    fn assert_parse(input: &str, build: impl FnOnce(&ExprBuilder)) {
        let exprs = parse_expr(input);
        let expected = ExprBuilder::default();
        build(&expected);
        assert_eq!(exprs, *expected.exprs.borrow(), "input: {input}");
    }

    #[test]
    fn simple_binop() {
        assert_parse("1 * 2", |eb| {
            eb.mul(eb.int(1), eb.int(2));
        });

        assert_parse("a % 123", |eb| {
            eb.modu(eb.ident("a"), eb.int(123));
        });

        assert_parse("1.2 / foo", |eb| {
            eb.div(eb.float(1.2), eb.ident("foo"));
        });

        assert_parse("1 + 2", |eb| {
            eb.add(eb.int(1), eb.int(2));
        });

        assert_parse("1 - 2", |eb| {
            eb.sub(eb.int(1), eb.int(2));
        });
    }

    #[test]
    fn binop_prec() {
        // `*` over `+` and `-`
        assert_parse("1 + 2 * 3 - 4", |eb| {
            eb.sub(eb.add(eb.int(1), eb.mul(eb.int(2), eb.int(3))), eb.int(4));
        });

        // left-assoc. addition and multiplication
        assert_parse("1 + 2 + 3 * 4 * 5", |eb| {
            eb.add(
                eb.add(eb.int(1), eb.int(2)),
                eb.mul(eb.mul(eb.int(3), eb.int(4)), eb.int(5)),
            );
        });

        // right-assoc. assignment
        assert_parse("a = b += c *= d -= e -= f", |eb| {
            eb.assign(
                eb.ident("a"),
                eb.add_assign(
                    eb.ident("b"),
                    eb.mul_assign(
                        eb.ident("c"),
                        eb.sub_assign(eb.ident("d"), eb.sub_assign(eb.ident("e"), eb.ident("f"))),
                    ),
                ),
            );
        });
    }

    #[test]
    fn atomic() {
        assert_parse("name", |eb| {
            eb.ident("name");
        });
        assert_parse("42", |eb| {
            eb.int(42);
        });
        assert_parse("3.5", |eb| {
            eb.float(3.5);
        });
        assert_parse("\"hello\"", |eb| {
            eb.string("hello");
        });
    }

    #[test]
    fn mul_add() {
        assert_parse("a * b / c % d", |eb| {
            eb.modu(
                eb.div(eb.mul(eb.ident("a"), eb.ident("b")), eb.ident("c")),
                eb.ident("d"),
            );
        });
        assert_parse("a + b - c + d", |eb| {
            eb.add(
                eb.sub(eb.add(eb.ident("a"), eb.ident("b")), eb.ident("c")),
                eb.ident("d"),
            );
        });
    }

    #[test]
    fn shift_eq_prec() {
        assert_parse("a + b << c * d", |eb| {
            eb.lshift(
                eb.add(eb.ident("a"), eb.ident("b")),
                eb.mul(eb.ident("c"), eb.ident("d")),
            );
        });
        assert_parse("a >> b < c <= d > e >= f", |eb| {
            eb.gt_eq(
                eb.gt(
                    eb.lt_eq(
                        eb.lt(eb.rshift(eb.ident("a"), eb.ident("b")), eb.ident("c")),
                        eb.ident("d"),
                    ),
                    eb.ident("e"),
                ),
                eb.ident("f"),
            );
        });
        assert_parse("a == b != c", |eb| {
            eb.neq(eb.eq(eb.ident("a"), eb.ident("b")), eb.ident("c"));
        });
    }

    #[test]
    fn logic_prec() {
        assert_parse("a || b && c || d", |eb| {
            eb.logic_or(
                eb.logic_or(eb.ident("a"), eb.logic_and(eb.ident("b"), eb.ident("c"))),
                eb.ident("d"),
            );
        });
        assert_parse("a && b && c", |eb| {
            eb.logic_and(eb.logic_and(eb.ident("a"), eb.ident("b")), eb.ident("c"));
        });
    }

    #[test]
    fn prefix_before_infix() {
        assert_parse("-a * !b + -c", |eb| {
            eb.add(
                eb.mul(
                    eb.unary(UnaryOp::Negate, eb.ident("a")),
                    eb.unary(UnaryOp::Not, eb.ident("b")),
                ),
                eb.unary(UnaryOp::Negate, eb.ident("c")),
            );
        });
        assert_parse("!!-value", |eb| {
            eb.unary(
                UnaryOp::Not,
                eb.unary(UnaryOp::Not, eb.unary(UnaryOp::Negate, eb.ident("value"))),
            );
        });
    }

    #[test]
    fn parens() {
        assert_parse("(a + b) * (c - d)", |eb| {
            eb.mul(
                eb.add(eb.ident("a"), eb.ident("b")),
                eb.sub(eb.ident("c"), eb.ident("d")),
            );
        });
        assert_parse("a - (b - c)", |eb| {
            eb.sub(eb.ident("a"), eb.sub(eb.ident("b"), eb.ident("c")));
        });
        assert_parse("((value))", |eb| {
            eb.ident("value");
        });
    }

    #[test]
    fn postfix_before_prefix() {
        assert_parse("items[i + 1] * -values[j]", |eb| {
            eb.mul(
                eb.index(eb.ident("items"), eb.add(eb.ident("i"), eb.int(1))),
                eb.unary(UnaryOp::Negate, eb.index(eb.ident("values"), eb.ident("j"))),
            );
        });
        assert_parse("matrix[row][column]", |eb| {
            eb.index(
                eb.index(eb.ident("matrix"), eb.ident("row")),
                eb.ident("column"),
            );
        });
    }

    #[test]
    fn assign() {
        assert_parse("a = b = c + d * e", |eb| {
            eb.assign(
                eb.ident("a"),
                eb.assign(
                    eb.ident("b"),
                    eb.add(eb.ident("c"), eb.mul(eb.ident("d"), eb.ident("e"))),
                ),
            );
        });
        assert_parse("a <<= b >>= c", |eb| {
            eb.binop(
                eb.ident("a"),
                BinaryOp::LshiftAssign,
                eb.binop(eb.ident("b"), BinaryOp::RshiftAssign, eb.ident("c")),
            );
        });
        assert_parse("a /= b %= c", |eb| {
            eb.div_assign(eb.ident("a"), eb.modu_assign(eb.ident("b"), eb.ident("c")));
        });
    }

    #[test]
    fn call() {
        assert_parse("foo()", |eb| {
            eb.call(eb.ident("foo"), Vec::new());
        });
        assert_parse("foo(a, 1, 3.0)", |eb| {
            eb.call(
                eb.ident("foo"),
                vec![eb.ident("a"), eb.int(1), eb.float(3.0)],
            );
        });
        assert_parse("foo(bar())", |eb| {
            eb.call(eb.ident("foo"), vec![eb.call(eb.ident("bar"), Vec::new())]);
        });
        assert_parse("foo(bar())(1)", |eb| {
            eb.call(
                eb.call(eb.ident("foo"), vec![eb.call(eb.ident("bar"), Vec::new())]),
                vec![eb.int(1)],
            );
        });
    }
}
