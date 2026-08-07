use crate::{
    Span,
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

            let Some((op, lbp, rbp)) = self.peek().and_then(|token| token.value.infix_bp()) else {
                break;
            };

            if lbp < min_bp {
                break;
            }

            self.next();
            let rhs = self.parse_expr_bp(rbp);
            lhs = self.ast.push_expr(
                Expr::Binary { lhs, op, rhs },
                self.ast.expr_span(lhs) + self.ast.expr_span(rhs),
            );
        }

        lhs
    }

    fn parse_expr_prefix(&mut self) -> ExprId {
        let Some(Token {
            value: tok,
            span: start,
        }) = self.next().cloned()
        else {
            panic!("expected expression, found end of input");
        };
        if let Some((op, bp)) = tok.prefix_bp() {
            let operand = self.parse_expr_bp(bp);
            return self.ast.push_expr(
                Expr::Unary { op, operand },
                start + self.ast.expr_span(operand),
            );
        }

        match tok {
            TokenKind::Ident(s) => self.ast.push_expr(Expr::Ident(s), start), // TODO: handle types
            TokenKind::Integer(n) => self.ast.push_expr(Expr::Integer(n), start),
            TokenKind::Decimal(f) => self.ast.push_expr(Expr::Decimal(f), start),
            TokenKind::String(s) => self.ast.push_expr(Expr::String(s), start),
            TokenKind::Punct(Punct::Lparen) => self.parse_expr_parens(start),
            _ => todo!("invalid token for expression: {tok:?}"),
        }
    }

    fn parse_expr_postfix(&mut self, lhs: ExprId, min_bp: u8) -> Option<ExprId> {
        const POSTFIX_BP: u8 = 30;

        if POSTFIX_BP < min_bp {
            return None;
        }

        match self.peek().map(|token| &token.value) {
            Some(TokenKind::Punct(Punct::Lsquare)) => {
                // `[]`: Array subscripting
                let start = self.next().expect("index opener disappeared").span;
                let ind = self.parse_expr_bp(0);
                let Some(Token {
                    value: tok,
                    span: end,
                }) = self.next()
                else {
                    panic!("expr: expected Rsquare but found end of input");
                };
                let end = *end;
                assert!(
                    matches!(tok, &TokenKind::Punct(Punct::Rsquare)),
                    "expr: expected Rsquare but found {tok:?}"
                );
                Some(
                    self.ast
                        .push_expr(Expr::GetIndex { obj: lhs, ind }, start + end),
                )
            }
            Some(TokenKind::Punct(Punct::Lparen)) => {
                // `()`: Function call
                let start = self.next().expect("call opener disappeared").span;
                let mut args = Vec::new();

                if !matches!(
                    self.peek().map(|token| &token.value),
                    Some(TokenKind::Punct(Punct::Rparen))
                ) {
                    loop {
                        args.push(self.parse_expr_bp(0));
                        match self.peek().map(|token| &token.value) {
                            Some(TokenKind::Punct(Punct::Comma)) => {
                                self.next();
                            }
                            Some(TokenKind::Punct(Punct::Rparen)) => {
                                break;
                            }
                            _ => {
                                panic!(
                                    "expected Comma or Rparen but found {:?}",
                                    self.peek().map(|token| &token.value)
                                )
                            }
                        }
                    }
                }

                let Some(Token {
                    value: tok,
                    span: end,
                }) = self.next()
                else {
                    panic!("expr: expected Rparen but found end of input");
                };
                let end = *end;
                assert!(
                    matches!(tok, &TokenKind::Punct(Punct::Rparen)),
                    "expr: expected Rparen but found {tok:?}"
                );
                Some(
                    self.ast
                        .push_expr(Expr::Call { callee: lhs, args }, start + end),
                )
            }
            _ => None,
        }
    }

    fn parse_expr_parens(&mut self, _start: Span) -> ExprId {
        let e = self.parse_expr_bp(0);
        let Some(tok) = self.next().map(|token| &token.value) else {
            panic!("expr: expected Rparen but found end of input");
        };
        assert!(
            matches!(tok, &TokenKind::Punct(Punct::Rparen)),
            "expr: expected Rparen but found {tok:?}"
        );
        e
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::ast::tests::AstBuilder;

    fn assert_parse(input: &str, build: impl FnOnce(&AstBuilder)) {
        let tokens = crate::lex::tests::lex(input);
        let mut parser = Parser::new(&tokens);
        let ast = AstBuilder::default();
        build(&ast);
        parser.parse_expr();
        ast.assert_matches(&parser.ast);
    }

    #[test]
    fn simple_binop() {
        assert_parse("1 * 2", |ast| {
            ast.expr_mul(ast.expr_int(1), ast.expr_int(2));
        });

        assert_parse("a % 123", |ast| {
            ast.expr_mod(ast.expr_ident("a"), ast.expr_int(123));
        });

        assert_parse("1.2 / foo", |ast| {
            ast.expr_div(ast.expr_float(1.2), ast.expr_ident("foo"));
        });

        assert_parse("1 + 2", |ast| {
            ast.expr_add(ast.expr_int(1), ast.expr_int(2));
        });

        assert_parse("1 - 2", |ast| {
            ast.expr_sub(ast.expr_int(1), ast.expr_int(2));
        });
    }

    #[test]
    fn binop_prec() {
        // `*` over `+` and `-`
        assert_parse("1 + 2 * 3 - 4", |ast| {
            ast.expr_sub(
                ast.expr_add(
                    ast.expr_int(1),
                    ast.expr_mul(ast.expr_int(2), ast.expr_int(3)),
                ),
                ast.expr_int(4),
            );
        });

        // left-assoc. addition and multiplication
        assert_parse("1 + 2 + 3 * 4 * 5", |ast| {
            ast.expr_add(
                ast.expr_add(ast.expr_int(1), ast.expr_int(2)),
                ast.expr_mul(
                    ast.expr_mul(ast.expr_int(3), ast.expr_int(4)),
                    ast.expr_int(5),
                ),
            );
        });

        // right-assoc. assignment
        assert_parse("a = b += c *= d -= e -= f", |ast| {
            ast.expr_set(
                ast.expr_ident("a"),
                ast.expr_binary(
                    ast.expr_ident("b"),
                    BinaryOp::AddAssign,
                    ast.expr_binary(
                        ast.expr_ident("c"),
                        BinaryOp::MulAssign,
                        ast.expr_binary(
                            ast.expr_ident("d"),
                            BinaryOp::SubAssign,
                            ast.expr_binary(
                                ast.expr_ident("e"),
                                BinaryOp::SubAssign,
                                ast.expr_ident("f"),
                            ),
                        ),
                    ),
                ),
            );
        });
    }

    #[test]
    fn atomic() {
        assert_parse("name", |ast| {
            ast.expr_ident("name");
        });
        assert_parse("42", |ast| {
            ast.expr_int(42);
        });
        assert_parse("3.5", |ast| {
            ast.expr_float(3.5);
        });
        assert_parse("\"hello\"", |ast| {
            ast.expr_str("hello");
        });
    }

    #[test]
    fn mul_add() {
        assert_parse("a * b / c % d", |ast| {
            ast.expr_mod(
                ast.expr_div(
                    ast.expr_mul(ast.expr_ident("a"), ast.expr_ident("b")),
                    ast.expr_ident("c"),
                ),
                ast.expr_ident("d"),
            );
        });
        assert_parse("a + b - c + d", |ast| {
            ast.expr_add(
                ast.expr_sub(
                    ast.expr_add(ast.expr_ident("a"), ast.expr_ident("b")),
                    ast.expr_ident("c"),
                ),
                ast.expr_ident("d"),
            );
        });
    }

    #[test]
    fn shift_eq_prec() {
        assert_parse("a + b << c * d", |ast| {
            ast.expr_binary(
                ast.expr_add(ast.expr_ident("a"), ast.expr_ident("b")),
                BinaryOp::Lshift,
                ast.expr_mul(ast.expr_ident("c"), ast.expr_ident("d")),
            );
        });
        assert_parse("a >> b < c <= d > e >= f", |ast| {
            ast.expr_binary(
                ast.expr_binary(
                    ast.expr_binary(
                        ast.expr_binary(
                            ast.expr_binary(
                                ast.expr_ident("a"),
                                BinaryOp::Rshift,
                                ast.expr_ident("b"),
                            ),
                            BinaryOp::Lt,
                            ast.expr_ident("c"),
                        ),
                        BinaryOp::LtEq,
                        ast.expr_ident("d"),
                    ),
                    BinaryOp::Gt,
                    ast.expr_ident("e"),
                ),
                BinaryOp::GtEq,
                ast.expr_ident("f"),
            );
        });
        assert_parse("a == b != c", |ast| {
            ast.expr_binary(
                ast.expr_eq(ast.expr_ident("a"), ast.expr_ident("b")),
                BinaryOp::Neq,
                ast.expr_ident("c"),
            );
        });
    }

    #[test]
    fn logic_prec() {
        assert_parse("a || b && c || d", |ast| {
            ast.expr_binary(
                ast.expr_binary(
                    ast.expr_ident("a"),
                    BinaryOp::LogicOr,
                    ast.expr_binary(ast.expr_ident("b"), BinaryOp::LogicAnd, ast.expr_ident("c")),
                ),
                BinaryOp::LogicOr,
                ast.expr_ident("d"),
            );
        });
        assert_parse("a && b && c", |ast| {
            ast.expr_binary(
                ast.expr_binary(ast.expr_ident("a"), BinaryOp::LogicAnd, ast.expr_ident("b")),
                BinaryOp::LogicAnd,
                ast.expr_ident("c"),
            );
        });
    }

    #[test]
    fn prefix_before_infix() {
        assert_parse("-a * !b + -c", |ast| {
            ast.expr_add(
                ast.expr_mul(
                    ast.expr_neg(ast.expr_ident("a")),
                    ast.expr_not(ast.expr_ident("b")),
                ),
                ast.expr_neg(ast.expr_ident("c")),
            );
        });
        assert_parse("!!-value", |ast| {
            ast.expr_not(ast.expr_not(ast.expr_neg(ast.expr_ident("value"))));
        });
    }

    #[test]
    fn parens() {
        assert_parse("(a + b) * (c - d)", |ast| {
            ast.expr_mul(
                ast.expr_add(ast.expr_ident("a"), ast.expr_ident("b")),
                ast.expr_sub(ast.expr_ident("c"), ast.expr_ident("d")),
            );
        });
        assert_parse("a - (b - c)", |ast| {
            ast.expr_sub(
                ast.expr_ident("a"),
                ast.expr_sub(ast.expr_ident("b"), ast.expr_ident("c")),
            );
        });
        assert_parse("((value))", |ast| {
            ast.expr_ident("value");
        });
    }

    #[test]
    fn postfix_before_prefix() {
        assert_parse("items[i + 1] * -values[j]", |ast| {
            ast.expr_mul(
                ast.expr_ind(
                    ast.expr_ident("items"),
                    ast.expr_add(ast.expr_ident("i"), ast.expr_int(1)),
                ),
                ast.expr_neg(ast.expr_ind(ast.expr_ident("values"), ast.expr_ident("j"))),
            );
        });
        assert_parse("matrix[row][column]", |ast| {
            ast.expr_ind(
                ast.expr_ind(ast.expr_ident("matrix"), ast.expr_ident("row")),
                ast.expr_ident("column"),
            );
        });
    }

    #[test]
    fn assign() {
        assert_parse("a = b = c + d * e", |ast| {
            ast.expr_set(
                ast.expr_ident("a"),
                ast.expr_set(
                    ast.expr_ident("b"),
                    ast.expr_add(
                        ast.expr_ident("c"),
                        ast.expr_mul(ast.expr_ident("d"), ast.expr_ident("e")),
                    ),
                ),
            );
        });
        assert_parse("a <<= b >>= c", |ast| {
            ast.expr_binary(
                ast.expr_ident("a"),
                BinaryOp::LshiftAssign,
                ast.expr_binary(
                    ast.expr_ident("b"),
                    BinaryOp::RshiftAssign,
                    ast.expr_ident("c"),
                ),
            );
        });
        assert_parse("a /= b %= c", |ast| {
            ast.expr_binary(
                ast.expr_ident("a"),
                BinaryOp::DivAssign,
                ast.expr_binary(
                    ast.expr_ident("b"),
                    BinaryOp::ModAssign,
                    ast.expr_ident("c"),
                ),
            );
        });
    }

    #[test]
    fn call() {
        assert_parse("foo()", |ast| {
            ast.expr_call(ast.expr_ident("foo"), &[]);
        });
        assert_parse("foo(a, 1, 3.0)", |ast| {
            ast.expr_call(
                ast.expr_ident("foo"),
                &[ast.expr_ident("a"), ast.expr_int(1), ast.expr_float(3.0)],
            );
        });
        assert_parse("foo(bar())", |ast| {
            ast.expr_call(
                ast.expr_ident("foo"),
                &[ast.expr_call(ast.expr_ident("bar"), &[])],
            );
        });
        assert_parse("foo(bar())(1)", |ast| {
            ast.expr_call(
                ast.expr_call(
                    ast.expr_ident("foo"),
                    &[ast.expr_call(ast.expr_ident("bar"), &[])],
                ),
                &[ast.expr_int(1)],
            );
        });
    }
}
