use super::Parser;
use crate::lex::token::{Punct, TokenKind};
use crate::parse::ast::{Stmt, StmtId};
use crate::{Span, Spanned};

impl Parser<'_> {
    pub(super) fn parse_stmt(&mut self) -> StmtId {
        if let Some(decl) = self.try_parse_decl() {
            return self.push_stmt(Stmt::Decl(decl), self.ast.decl(decl).span);
        }

        let Some(Spanned {
            value: tok,
            span: start,
        }) = self.peek()
        else {
            panic!("unexpected eof");
        };
        let start = *start;

        if *tok == TokenKind::Punct(Punct::Lcurly) {
            self.next();
            let mut stmts = Vec::new();
            while self.pos < self.tokens.len()
                && self.peek().map(|t| &t.value) != Some(&TokenKind::Punct(Punct::Rcurly))
            {
                stmts.push(self.parse_stmt());
            }
            let Some(next) = self.next() else {
                panic!("unexpected eof");
            };
            let Spanned {
                value: TokenKind::Punct(Punct::Rcurly),
                span: end,
            } = next
            else {
                panic!("not rcurly");
            };
            let end = *end;
            return self.push_stmt(Stmt::Block(stmts), start + end);
        }

        let expr = self.parse_expr();
        let Some(next) = self.next() else {
            panic!("unexpected eof");
        };
        let Spanned {
            value: TokenKind::Punct(Punct::Semicolon),
            span: end,
        } = next
        else {
            panic!("expected semicolon, got {:?}", next.value);
        };
        let end = *end;
        self.push_stmt(Stmt::Expr(expr), start + end)
    }

    fn push_stmt(&mut self, stmt: Stmt, span: Span) -> StmtId {
        self.ast.stmts.push(Spanned { value: stmt, span });
        #[allow(clippy::cast_possible_truncation)]
        StmtId((self.ast.stmts.len() - 1) as u32)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lex::token::Token,
        parse::ast::{BinaryOp, Decl, DeclId, Expr, ExprId, Type},
    };

    use super::*;

    fn lex(input: &str) -> Vec<Token> {
        crate::lex::lex(7, &(String::from(input) + "\n")).0.unwrap()
    }

    #[test]
    fn parse_stmt_expr() {
        let tokens = lex("a += b * c;");
        let mut parser = Parser::new(&tokens);
        assert_eq!(parser.parse_stmt(), StmtId(0));
        assert_eq!(
            parser.ast.stmts[0].span,
            Span {
                file: 7,
                start: 0,
                end: 11
            }
        );
        assert_eq!(parser.ast.stmts[0].value, Stmt::Expr(ExprId(4)));
        assert_eq!(parser.ast.exprs[0].value, Expr::Ident("a".into()));
        assert_eq!(parser.ast.exprs[1].value, Expr::Ident("b".into()));
        assert_eq!(parser.ast.exprs[2].value, Expr::Ident("c".into()));
        assert_eq!(
            parser.ast.exprs[3].value,
            Expr::Binary {
                lhs: ExprId(1),
                rhs: ExprId(2),
                op: BinaryOp::Mul
            }
        );
        assert_eq!(
            parser.ast.exprs[4].value,
            Expr::Binary {
                lhs: ExprId(0),
                rhs: ExprId(3),
                op: BinaryOp::AddAssign
            }
        );
    }

    #[test]
    fn parse_stmt_decl() {
        let tokens = lex("int a = b * c; float d;");
        let mut parser = Parser::new(&tokens);
        // `int a = b * c;`
        assert_eq!(parser.parse_stmt(), StmtId(0));
        assert_eq!(
            parser.ast.stmts[0].span,
            Span {
                file: 7,
                start: 0,
                end: 14
            }
        );
        assert_eq!(parser.ast.stmts[0].value, Stmt::Decl(DeclId(0)));
        // `int a = b * c;`
        assert_eq!(
            parser.ast.decls[0].value,
            Decl::Variable {
                typ: Type::Int,
                name: "a".into(),
                init: Some(ExprId(2))
            }
        );
        // `b`
        assert_eq!(parser.ast.exprs[0].value, Expr::Ident("b".into()));
        // `c`
        assert_eq!(parser.ast.exprs[1].value, Expr::Ident("c".into()));
        // `b * c`
        assert_eq!(
            parser.ast.exprs[2].value,
            Expr::Binary {
                lhs: ExprId(0),
                rhs: ExprId(1),
                op: BinaryOp::Mul
            }
        );
        // `float d;`
        assert_eq!(parser.parse_stmt(), StmtId(1));
        assert_eq!(parser.ast.stmts[1].value, Stmt::Decl(DeclId(1)));
        assert_eq!(
            parser.ast.stmts[1].span,
            Span {
                file: 7,
                start: 15,
                end: 23
            }
        );
        // `float d;`
        assert_eq!(
            parser.ast.decls[1].value,
            Decl::Variable {
                typ: Type::Float,
                name: "d".into(),
                init: None
            }
        );
    }
}
