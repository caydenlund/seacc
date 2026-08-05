use super::Parser;
use crate::parse::ast::{Item, ItemId};
use crate::{Span, Spanned};

impl Parser<'_> {
    pub(super) fn parse_item(&mut self) -> ItemId {
        if let Some(decl) = self.try_parse_decl() {
            return self.push_item(Item::Decl(decl), self.ast.decl(decl).span);
        }

        todo!("function definition")
    }

    fn push_item(&mut self, item: Item, span: Span) -> ItemId {
        self.ast.items.push(Spanned { value: item, span });
        #[allow(clippy::cast_possible_truncation)]
        ItemId((self.ast.items.len() - 1) as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::token::Token;
    use crate::parse::ast::{
        BinaryOp, Decl, DeclId, Expr, ExprId, FunctionDecl, Param, Stmt, StmtId, Type,
    };

    fn lex(input: &str) -> Vec<Token> {
        crate::lex::lex(7, &(String::from(input) + "\n")).0.unwrap()
    }

    #[test]
    fn decl() {
        let tokens = lex("int a; float b = 1.0; int foo(); float bar(int x, int, int);");
        let mut parser = Parser::new(&tokens);
        // `int a;`
        assert_eq!(parser.parse_item(), ItemId(0));
        assert_eq!(parser.ast.items[0].value, Item::Decl(DeclId(0)));
        assert_eq!(
            parser.ast.decls[0].value,
            Decl::Variable {
                typ: Type::Int,
                name: "a".into(),
                init: None
            }
        );
        // `float b = 1.0;`
        assert_eq!(parser.parse_item(), ItemId(1));
        assert_eq!(parser.ast.items[1].value, Item::Decl(DeclId(1)));
        assert_eq!(
            parser.ast.decls[1].value,
            Decl::Variable {
                typ: Type::Float,
                name: "b".into(),
                init: Some(ExprId(0)),
            }
        );
        // `1.0`
        assert_eq!(parser.ast.exprs[0].value, Expr::Decimal(1.0));
        // `int foo();`
        assert_eq!(parser.parse_item(), ItemId(2));
        assert_eq!(parser.ast.items[2].value, Item::Decl(DeclId(2)));
        assert_eq!(
            parser.ast.decls[2].value,
            Decl::Function(FunctionDecl {
                ret_typ: Type::Int,
                name: "foo".into(),
                params: Vec::new(),
                variadic: false,
            })
        );
        // `float bar(int x, int, int);`
        assert_eq!(parser.parse_item(), ItemId(3));
        assert_eq!(parser.ast.items[3].value, Item::Decl(DeclId(3)));
        assert_eq!(
            parser.ast.decls[3].value,
            Decl::Function(FunctionDecl {
                ret_typ: Type::Float,
                name: "bar".into(),
                params: vec![
                    Param {
                        typ: Type::Int,
                        name: Some("x".into())
                    },
                    Param {
                        typ: Type::Int,
                        name: None
                    },
                    Param {
                        typ: Type::Int,
                        name: None
                    }
                ],
                variadic: false,
            })
        );
    }

    #[test]
    fn func_def() {
        let tokens = lex("int main(int argc) { int a = 2; int b; b = a + argc; return a * b; }");
        let mut parser = Parser::new(&tokens);
        // `int main() { /* ... */ }`
        assert_eq!(parser.parse_item(), ItemId(0));
        assert_eq!(
            parser.ast.items[0].value,
            Item::FuncDef {
                decl: FunctionDecl {
                    ret_typ: Type::Int,
                    name: "main".into(),
                    params: vec![Param {
                        typ: Type::Int,
                        name: Some("argc".into())
                    }],
                    variadic: false
                },
                body: StmtId(4)
            }
        );
        // `int a = 1;`
        assert_eq!(parser.ast.stmts[0].value, Stmt::Decl(DeclId(0)));
        assert_eq!(
            parser.ast.decls[0].value,
            Decl::Variable {
                typ: Type::Int,
                name: "a".into(),
                init: Some(ExprId(0))
            }
        );
        // `1`
        assert_eq!(parser.ast.exprs[0].value, Expr::Integer(2));
        // `int b;`
        assert_eq!(parser.ast.stmts[1].value, Stmt::Decl(DeclId(1)));
        assert_eq!(
            parser.ast.decls[1].value,
            Decl::Variable {
                typ: Type::Int,
                name: "b".into(),
                init: None
            }
        );
        // `b = a + argc;`
        assert_eq!(parser.ast.stmts[2].value, Stmt::Expr(ExprId(3)));
        // `a`
        assert_eq!(parser.ast.exprs[1].value, Expr::Ident("a".into()));
        // `argc`
        assert_eq!(parser.ast.exprs[2].value, Expr::Ident("argc".into()));
        // `a + argc`
        assert_eq!(
            parser.ast.exprs[3].value,
            Expr::Binary {
                lhs: ExprId(1),
                op: BinaryOp::Add,
                rhs: ExprId(2)
            }
        );
        // `return a * b;`
        assert_eq!(parser.ast.stmts[3].value, Stmt::Return(Some(ExprId(6))));
        // `a`
        assert_eq!(parser.ast.exprs[4].value, Expr::Ident("a".into()));
        // `b`
        assert_eq!(parser.ast.exprs[5].value, Expr::Ident("b".into()));
        // `a * b`
        assert_eq!(
            parser.ast.exprs[6].value,
            Expr::Binary {
                lhs: ExprId(4),
                op: BinaryOp::Mul,
                rhs: ExprId(5)
            }
        );
        // `{ /* ... */ }`
        assert_eq!(
            parser.ast.stmts[4].value,
            Stmt::Block(vec![StmtId(0), StmtId(1), StmtId(2), StmtId(3)])
        );
    }
}
