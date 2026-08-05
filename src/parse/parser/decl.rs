use super::Parser;
use crate::{
    Span, Spanned,
    lex::token::{Punct, TokenKind},
    parse::ast::{Decl, DeclId},
};

impl Parser<'_> {
    pub(super) fn try_parse_decl(&mut self) -> Option<DeclId> {
        // TODO: multiple variables
        // TODO: function
        // TODO: typedef
        // TODO: struct
        // TODO: union
        // TODO: enum

        if let Some(Spanned {
            value: typ,
            span: start,
        }) = self.try_parse_type()
        {
            let next = self.next();
            let Some(TokenKind::Ident(name)) = next.map(|t| &t.value) else {
                panic!("expected Ident but found {next:?}")
            };
            let name = name.clone();

            match self.peek().map(|t| &t.value) {
                Some(TokenKind::Punct(Punct::Semicolon)) => {
                    let end = self.next().unwrap().span;
                    return Some(self.ast.push_decl(
                        Decl::Variable {
                            typ,
                            name,
                            init: None,
                        },
                        start + end,
                    ));
                }
                Some(TokenKind::Punct(Punct::Eq)) => {
                    self.pos += 1;
                    let init = Some(self.parse_expr());
                    let next = self.next();
                    let Some(Spanned {
                        value: TokenKind::Punct(Punct::Semicolon),
                        span: end,
                    }) = next
                    else {
                        panic!("expected Semicolon, found {next:?}")
                    };
                    let end = *end;
                    return Some(
                        self.ast
                            .push_decl(Decl::Variable { typ, name, init }, start + end),
                    );
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
                    return Some(
                        self.ast
                            .push_decl(Decl::Function(decl), function_span + end),
                    );
                }
                t => panic!("unexpected token in decl: {t:?}"),
            }
        }

        None
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
        let x = ast.decl_var(Type::Int, "x", None);
        assert_eq!(parser.try_parse_decl(), Some(x));
        // `float y = 1.0;`
        let y_init = ast.expr_float(1.0);
        let y = ast.decl_var(Type::Float, "y", Some(y_init));
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
}
