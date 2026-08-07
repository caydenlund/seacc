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
    use super::parse;
    use crate::lex::tests::lex;
    use crate::parse::ast::tests::AstBuilder;
    use crate::parse::ast::{FunctionDecl, Type};

    #[test]
    fn parses_multiple_top_level_items() {
        let tokens = lex("float scale; int main() { return 42; }");
        let ast = parse(&tokens).ast;
        let ab = AstBuilder::default();

        let scale = ab.decl_vars(Type::Float, &[("scale", None)]);
        ab.item_decl(scale);
        let value = ab.expr_int(42);
        let ret = ab.stmt_return(Some(value));
        ab.item_func(
            FunctionDecl {
                ret_typ: Type::Int,
                name: "main".into(),
                params: vec![],
                variadic: false,
            },
            &[ret],
        );
        ab.assert_matches(&ast);
    }
}
