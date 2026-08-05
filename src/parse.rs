use crate::lex::token::Token;

pub mod ast;

mod parser;
pub use parser::ParsedOutput;

#[must_use]
pub fn parse(tokens: &[Token]) -> ParsedOutput {
    parser::Parser::new(tokens).parse()
}

#[cfg(test)]
mod tests {
    use crate::parse::ast::{Decl, DeclId, Expr, ExprId, Item, Stmt, Type};

    #[test]
    fn parses_multiple_top_level_items() {
        let tokens = crate::lex::lex(3, "float scale; int main() { return 42; }\n")
            .0
            .expect("should tokenize");
        let output = super::parse(&tokens);

        assert_eq!(output.ast.items.len(), 2);
        assert_eq!(output.ast.items[0].value, Item::Decl(DeclId(0)));
        assert_eq!(
            output.ast.decls[0].value,
            Decl::Variable {
                typ: Type::Float,
                name: "scale".into(),
                init: None,
            }
        );
        assert!(matches!(output.ast.items[1].value, Item::FuncDef { .. }));
        assert_eq!(output.ast.stmts[0].value, Stmt::Return(Some(ExprId(0))));
        assert_eq!(output.ast.exprs[0].value, Expr::Integer(42));
    }
}
