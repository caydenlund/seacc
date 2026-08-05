use super::Parser;
use crate::lex::token::{Punct, TokenKind};
use crate::parse::ast::{Decl, Item, ItemId};

impl Parser<'_> {
    pub(super) fn parse_item(&mut self) -> ItemId {
        if let Some(decl) = self.try_parse_function_decl() {
            if self.peek().map(|token| &token.value) == Some(&TokenKind::Punct(Punct::Lcurly)) {
                self.next().expect("function body opener disappeared");
                let body = self.parse_stmts();
                let Some(closing_brace) = self.next() else {
                    panic!("expected Rcurly after function body");
                };
                assert!(
                    matches!(closing_brace.value, TokenKind::Punct(Punct::Rcurly)),
                    "expected Rcurly after function body, found {:?}",
                    closing_brace.value
                );
                let span = decl.span + closing_brace.span;
                return self.ast.push_item(
                    Item::FuncDef {
                        decl: decl.value,
                        body,
                    },
                    span,
                );
            }

            let Some(semicolon) = self.next() else {
                panic!("expected Semicolon after function declaration");
            };
            assert!(
                matches!(
                    semicolon.value,
                    crate::lex::token::TokenKind::Punct(crate::lex::token::Punct::Semicolon)
                ),
                "expected Semicolon after function declaration, found {:?}",
                semicolon.value
            );
            let span = decl.span + semicolon.span;
            let decl = self.ast.push_decl(Decl::Function(decl.value), span);
            return self.ast.push_item(Item::Decl(decl), span);
        }

        if let Some(decl) = self.try_parse_decl() {
            return self
                .ast
                .push_item(Item::Decl(decl), self.ast.decl_span(decl));
        }

        todo!("function definition")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::token::Token;
    use crate::parse::ast::tests::AstBuilder;
    use crate::parse::ast::{FunctionDecl, Param, Type};

    fn lex(input: &str) -> Vec<Token> {
        crate::lex::lex(7, &(String::from(input) + "\n")).0.unwrap()
    }

    #[test]
    fn decl() {
        let tokens = lex("int a; float b = 1.0; int foo(); float bar(int x, int, int);");
        let mut parser = Parser::new(&tokens);
        let ast = AstBuilder::default();
        let a = ast.decl_var(Type::Int, "a", None);
        let a_item = ast.item_decl(a);
        assert_eq!(parser.parse_item(), a_item);
        let b_init = ast.expr_float(1.0);
        let b = ast.decl_var(Type::Float, "b", Some(b_init));
        let b_item = ast.item_decl(b);
        assert_eq!(parser.parse_item(), b_item);
        let foo = ast.decl_fn(FunctionDecl {
            ret_typ: Type::Int,
            name: "foo".into(),
            params: vec![],
            variadic: false,
        });
        let foo_item = ast.item_decl(foo);
        assert_eq!(parser.parse_item(), foo_item);
        let bar = ast.decl_fn(FunctionDecl {
            ret_typ: Type::Float,
            name: "bar".into(),
            params: vec![
                Param {
                    typ: Type::Int,
                    name: Some("x".into()),
                },
                Param {
                    typ: Type::Int,
                    name: None,
                },
                Param {
                    typ: Type::Int,
                    name: None,
                },
            ],
            variadic: false,
        });
        let bar_item = ast.item_decl(bar);
        assert_eq!(parser.parse_item(), bar_item);
        ast.assert_matches(&parser.ast);
    }

    #[test]
    fn func_def() {
        let tokens = lex("int main(int argc) { int a = 2; int b; b = a + argc; return a * b; }");
        let mut parser = Parser::new(&tokens);
        let ast = AstBuilder::default();
        let two = ast.expr_int(2);
        let a = ast.decl_var(Type::Int, "a", Some(two));
        let a_stmt = ast.stmt_decl(a);
        let b = ast.decl_var(Type::Int, "b", None);
        let b_stmt = ast.stmt_decl(b);
        let assign = ast.expr_set(
            ast.expr_ident("b"),
            ast.expr_add(ast.expr_ident("a"), ast.expr_ident("argc")),
        );
        let assign_stmt = ast.stmt_expr(assign);
        let product = ast.expr_mul(ast.expr_ident("a"), ast.expr_ident("b"));
        let ret = ast.stmt_return(Some(product));
        let main = ast.item_func(
            FunctionDecl {
                ret_typ: Type::Int,
                name: "main".into(),
                params: vec![Param {
                    typ: Type::Int,
                    name: Some("argc".into()),
                }],
                variadic: false,
            },
            &[a_stmt, b_stmt, assign_stmt, ret],
        );
        assert_eq!(parser.parse_item(), main);
        ast.assert_matches(&parser.ast);
    }
}
