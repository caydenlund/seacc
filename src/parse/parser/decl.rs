use super::Parser;
use crate::{
    Spanned,
    lex::token::{Punct, TokenKind},
    parse::ast::{Decl, DeclId},
};

impl Parser<'_> {
    pub(super) fn try_parse_decl(&mut self) -> Option<DeclId> {
        // TODO: function
        // TODO: typedef
        // TODO: struct
        // TODO: union
        // TODO: enum

        let Spanned {
            value: typ,
            span: start,
        } = self.try_parse_type()?;
        let next = self.next();
        let Some(TokenKind::Ident(name)) = next.map(|t| &t.value) else {
            panic!("expected Ident but found {next:?}")
        };
        let name = name.clone();

        match self.peek().map(|t| &t.value) {
            Some(TokenKind::Punct(Punct::Semicolon | Punct::Comma | Punct::Eq)) => {
                let mut vars = Vec::new();
                let mut name = name;

                loop {
                    let init =
                        if self.peek().map(|t| &t.value) == Some(&TokenKind::Punct(Punct::Eq)) {
                            self.next();
                            Some(self.parse_expr())
                        } else {
                            None
                        };
                    vars.push((std::mem::take(&mut name), init));

                    match self.peek().map(|t| &t.value) {
                        Some(TokenKind::Punct(Punct::Comma)) => {
                            self.next();
                            let next = self.next();
                            let Some(Spanned {
                                value: TokenKind::Ident(next_name),
                                ..
                            }) = next
                            else {
                                panic!("expected Ident after Comma, found {next:?}")
                            };
                            name.push_str(next_name);
                        }
                        Some(TokenKind::Punct(Punct::Semicolon)) => {
                            let end = self.next().unwrap().span;
                            return Some(
                                self.ast
                                    .push_decl(Decl::Variables { typ, vars }, start + end),
                            );
                        }
                        token => {
                            panic!("expected Comma or Semicolon in declaration, found {token:?}")
                        }
                    }
                }
            }
            Some(TokenKind::Punct(Punct::Lparen)) => {
                self.pos = self
                    .pos
                    .checked_sub(2)
                    .expect("declaration position underflow");
                let Spanned {
                    value: decl,
                    span: function_span,
                } = self
                    .try_parse_function_decl()
                    .expect("function declaration disappeared");
                let Some(Spanned {
                    value: TokenKind::Punct(Punct::Semicolon),
                    span: end,
                }) = self.next()
                else {
                    panic!("expected Semicolon after function declaration")
                };
                let end = *end;
                Some(
                    self.ast
                        .push_decl(Decl::Function(decl), function_span + end),
                )
            }
            t => panic!("unexpected token in decl: {t:?}"),
        }
    }

    pub(super) fn try_parse_function_decl(
        &mut self,
    ) -> Option<Spanned<crate::parse::ast::FunctionDecl>> {
        let start_pos = self.pos;
        let Spanned {
            value: ret_typ,
            span: start,
        } = self.try_parse_type()?;
        let Some(Spanned {
            value: TokenKind::Ident(name),
            ..
        }) = self.next()
        else {
            self.pos = start_pos;
            return None;
        };
        let name = name.clone();
        if self.peek().map(|token| &token.value) != Some(&TokenKind::Punct(Punct::Lparen)) {
            self.pos = start_pos;
            return None;
        }
        self.next();

        let mut params = Vec::new();
        if self.peek().map(|token| &token.value) != Some(&TokenKind::Punct(Punct::Rparen)) {
            loop {
                let Some(Spanned { value: typ, .. }) = self.try_parse_type() else {
                    panic!("expected parameter type")
                };
                let name = match self.peek().map(|token| &token.value) {
                    Some(TokenKind::Ident(_)) => match self.next().unwrap().value.clone() {
                        TokenKind::Ident(name) => Some(name),
                        _ => unreachable!(),
                    },
                    _ => None,
                };
                params.push(crate::parse::ast::Param { typ, name });

                match self.peek().map(|token| &token.value) {
                    Some(TokenKind::Punct(Punct::Comma)) => {
                        self.next();
                    }
                    Some(TokenKind::Punct(Punct::Rparen)) => break,
                    token => panic!("expected Comma or Rparen in parameter list, found {token:?}"),
                }
            }
        }

        let Some(Spanned {
            value: TokenKind::Punct(Punct::Rparen),
            span: end,
        }) = self.next()
        else {
            panic!("expected Rparen after function parameters")
        };
        Some(Spanned {
            value: crate::parse::ast::FunctionDecl {
                ret_typ,
                name,
                params,
                variadic: false,
            },
            span: start + *end,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::tests::lex;
    use crate::parse::ast::Type;
    use crate::parse::ast::tests::AstBuilder;

    #[test]
    fn single_var_decl() {
        let tokens = lex("int x; float y = 1.0; x += 1; y = 2.0;");
        let ast = AstBuilder::default();
        let mut parser = Parser::new(&tokens);

        // `int x;`
        let x = ast.decl_vars(Type::Int, &[("x", None)]);
        assert_eq!(parser.try_parse_decl(), Some(x));
        // `float y = 1.0;`
        let y_init = ast.expr_float(1.0);
        let y = ast.decl_vars(Type::Float, &[("y", Some(y_init))]);
        assert_eq!(parser.try_parse_decl(), Some(y));
        // `x`
        assert_eq!(parser.try_parse_decl(), None);
        // (parser shouldn't have advanced)
        assert_eq!(
            parser.peek().map(|t| &t.value),
            Some(&TokenKind::Ident("x".into()))
        );
        ast.assert_matches(&parser.ast);
    }

    #[test]
    fn multi_var_decl() {
        let tokens = lex("int x, y, z; float a = 1.0, b = 2.0; int foo, bar = 0;");
        let ast = AstBuilder::default();
        let mut parser = Parser::new(&tokens);

        // `int x, y, z;`
        let xyz = ast.decl_vars(Type::Int, &[("x", None), ("y", None), ("z", None)]);
        assert_eq!(parser.try_parse_decl(), Some(xyz));

        // `float a = 1.0, b = 2.0;`
        let ab = ast.decl_vars(
            Type::Float,
            &[
                ("a", Some(ast.expr_float(1.0))),
                ("b", Some(ast.expr_float(2.0))),
            ],
        );
        assert_eq!(parser.try_parse_decl(), Some(ab));

        // `int foo, bar = 0;`
        let fb = ast.decl_vars(Type::Int, &[("foo", None), ("bar", Some(ast.expr_int(0)))]);
        assert_eq!(parser.try_parse_decl(), Some(fb));

        ast.assert_matches(&parser.ast);
    }
}
