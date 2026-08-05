use super::Parser;
use crate::lex::token::{Keyword, Punct, TokenKind};
use crate::parse::ast::{Stmt, StmtId};
use crate::{Span, Spanned};

impl Parser<'_> {
    pub(super) fn parse_stmt(&mut self) -> StmtId {
        if let Some(decl) = self.try_parse_decl() {
            return self
                .ast
                .push_stmt(Stmt::Decl(decl), self.ast.decl_span(decl));
        }

        let tok = self.peek().map(|t| &t.value);
        match tok {
            Some(TokenKind::Punct(Punct::Lcurly)) => self.parse_stmt_block(),
            Some(TokenKind::Keyword(Keyword::Return)) => self.parse_stmt_return(),
            Some(TokenKind::Keyword(
                kw @ (Keyword::If
                | Keyword::For
                | Keyword::While
                | Keyword::Do
                | Keyword::Break
                | Keyword::Switch
                | Keyword::Continue),
            )) => {
                todo!("implement {kw:?} stmts")
            }
            Some(_) => self.parse_stmt_expr(),
            None => panic!("unexpected eof"),
        }
    }

    pub(super) fn parse_stmts(&mut self) -> Vec<StmtId> {
        let mut stmts = Vec::new();
        while self.peek().map(|t| &t.value) != Some(&TokenKind::Punct(Punct::Rcurly)) {
            stmts.push(self.parse_stmt());
        }
        stmts
    }

    fn parse_stmt_block(&mut self) -> StmtId {
        let Some(Spanned {
            value: TokenKind::Punct(Punct::Lcurly),
            span: start,
        }) = self.next()
        else {
            panic!("stmt block didn't find Lcurly");
        };
        let start = *start;

        let stmts = self.parse_stmts();

        let Some(Spanned {
            value: TokenKind::Punct(Punct::Rcurly),
            span: end,
        }) = self.next()
        else {
            panic!("stmt block didn't find Rcurly");
        };
        let end = *end;

        self.ast.push_stmt(Stmt::Block(stmts), start + end)
    }

    fn parse_stmt_return(&mut self) -> StmtId {
        let Some(Spanned {
            value: TokenKind::Keyword(Keyword::Return),
            span: start,
        }) = self.next()
        else {
            panic!("return stmt didn't find Return");
        };
        let start = *start;

        let expr = self
            .peek()
            .map(|t| t.value.clone())
            .and_then(|t| (t != TokenKind::Punct(Punct::Semicolon)).then(|| self.parse_expr()));

        let Some(Spanned {
            value: TokenKind::Punct(Punct::Semicolon),
            span: end,
        }) = self.next()
        else {
            panic!("return stmt didn't find Semicolon");
        };
        let end = *end;

        self.ast.push_stmt(Stmt::Return(expr), start + end)
    }

    fn parse_stmt_expr(&mut self) -> StmtId {
        let expr = self.parse_expr();
        let Some(Spanned {
            value: TokenKind::Punct(Punct::Semicolon),
            span: end,
        }) = self.next()
        else {
            panic!("expr stmt didn't find Semicolon");
        };
        let end = *end;
        self.ast
            .push_stmt(Stmt::Expr(expr), self.ast.expr_span(expr) + end)
    }
}

#[cfg(test)]
mod tests {
    use crate::parse::ast::tests::AstBuilder;
    use crate::{
        lex::token::Token,
        parse::ast::{BinaryOp, Type},
    };

    use super::*;

    fn lex(input: &str) -> Vec<Token> {
        crate::lex::lex(7, &(String::from(input) + "\n")).0.unwrap()
    }

    #[test]
    fn parse_stmt_expr() {
        let tokens = lex("a += b * c;");
        let mut parser = Parser::new(&tokens);
        let ast = AstBuilder::default();
        let expr = ast.expr_binary(
            ast.expr_ident("a"),
            BinaryOp::AddAssign,
            ast.expr_mul(ast.expr_ident("b"), ast.expr_ident("c")),
        );
        let stmt = ast.stmt_expr(expr);
        assert_eq!(parser.parse_stmt(), stmt);
        assert_eq!(
            parser.ast.stmts[0].span,
            Span {
                file: 7,
                start: 0,
                end: 11
            }
        );
        ast.assert_matches(&parser.ast);
    }

    #[test]
    fn parse_stmt_decl() {
        let tokens = lex("int a = b * c; float d;");
        let mut parser = Parser::new(&tokens);
        let ast = AstBuilder::default();
        // `int a = b * c;`
        let init = ast.expr_mul(ast.expr_ident("b"), ast.expr_ident("c"));
        let a = ast.decl_var(Type::Int, "a", Some(init));
        let a_stmt = ast.stmt_decl(a);
        assert_eq!(parser.parse_stmt(), a_stmt);
        assert_eq!(
            parser.ast.stmts[0].span,
            Span {
                file: 7,
                start: 0,
                end: 14
            }
        );
        // `float d;`
        let d = ast.decl_var(Type::Float, "d", None);
        let d_stmt = ast.stmt_decl(d);
        assert_eq!(parser.parse_stmt(), d_stmt);
        assert_eq!(
            parser.ast.stmts[1].span,
            Span {
                file: 7,
                start: 15,
                end: 23
            }
        );
        ast.assert_matches(&parser.ast);
    }

    #[test]
    fn parse_return_without_a_value() {
        let tokens = lex("return;");
        let mut parser = Parser::new(&tokens);
        let ast = AstBuilder::default();

        assert_eq!(parser.parse_stmt(), ast.stmt_return(None));
        assert_eq!(
            parser.ast.stmts[0].span,
            Span {
                file: 7,
                start: 0,
                end: 7,
            }
        );
        ast.assert_matches(&parser.ast);
    }
}
