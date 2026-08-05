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
                    return Some(self.push_decl(
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
                    return Some(self.push_decl(Decl::Variable { typ, name, init }, start + end));
                }
                t => panic!("unexpected token in decl: {t:?}"),
            }
        }

        None
    }

    fn push_decl(&mut self, decl: Decl, span: Span) -> DeclId {
        self.ast.decls.push(Spanned { value: decl, span });
        #[allow(clippy::cast_possible_truncation)]
        DeclId((self.ast.decls.len() - 1) as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        lex::token::Token,
        parse::ast::{Expr, ExprId, Type},
    };

    fn lex(input: &str) -> Vec<Token> {
        crate::lex::lex(7, &(String::from(input) + "\n")).0.unwrap()
    }

    #[test]
    fn single_var_decl() {
        let tokens = lex("int x; float y = 1.0; x += 1; y = 2.0;");
        let mut parser = Parser::new(&tokens);
        // `int x;`
        assert_eq!(parser.try_parse_decl(), Some(DeclId(0)));
        assert_eq!(
            parser.ast.decls[0].value,
            Decl::Variable {
                typ: Type::Int,
                name: "x".into(),
                init: None
            }
        );
        // `float y = 1.0;`
        assert_eq!(parser.try_parse_decl(), Some(DeclId(1)));
        assert_eq!(
            parser.ast.decls[1].value,
            Decl::Variable {
                typ: Type::Float,
                name: "y".into(),
                init: Some(ExprId(0))
            }
        );
        assert_eq!(parser.ast.exprs[0].value, Expr::Decimal(1.0));
        // `x`
        assert_eq!(parser.try_parse_decl(), None);
        // (parser shouldn't have advanced)
        assert_eq!(
            parser.peek().map(|t| &t.value),
            Some(&TokenKind::Ident("x".into()))
        );
    }
}
