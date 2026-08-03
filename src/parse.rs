use crate::{Spanned, lex::token::Token};
use ast::Expr;

pub mod ast;

mod error;
pub use error::{ParseError, ParseErrorKind};

mod parser;

pub fn parse(tokens: &[Token]) -> Result<Vec<Spanned<Expr>>, Vec<ParseError>> {
    let parser = parser::Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::{
        self,
        token::{Punct, TokenKind},
    };
    use ast::Expr;

    fn parse_input(input: &str) -> Result<Vec<Expr>, Vec<ParseErrorKind>> {
        let (tokens, _) = lex::lex(0, input);
        Ok(parse(&tokens.expect("valid lexer input"))
            .map_err(|e| e.into_iter().map(|e| e.value).collect::<Vec<_>>())?
            .into_iter()
            .map(|e| e.value)
            .collect())
    }

    fn root_sexpr(exprs: &[Expr]) -> String {
        exprs
            .last()
            .expect("non-empty expression list")
            .sexpr(exprs)
    }

    #[test]
    fn parses_atoms() {
        assert_eq!(parse_input("42\n"), Ok(vec![Expr::Integer(42)]));
        assert_eq!(parse_input("name\n"), Ok(vec![Expr::Ident("name".into())]));
        assert_eq!(
            parse_input("\"hello\"\n"),
            Ok(vec![Expr::String("hello".into())])
        );
    }

    #[test]
    fn multiplication_binds_more_tightly_than_addition() {
        let exprs = parse_input("1 + 2 * 3\n").expect("parse succeeds");

        assert_eq!(root_sexpr(&exprs), "(Add 1 (Mul 2 3))");
    }

    #[test]
    fn parentheses_and_unary_operators_override_precedence() {
        let exprs = parse_input("-(1 + 2) * !flag\n").expect("parse succeeds");

        assert_eq!(
            root_sexpr(&exprs),
            "(Mul (Negate (Add 1 2)) (Not (Ident flag)))"
        );
    }

    #[test]
    fn rejects_incomplete_and_trailing_expressions() {
        assert_eq!(
            parse_input("1 +\n"),
            Err(vec![ParseErrorKind::UnexpectedToken(TokenKind::Punct(
                Punct::Plus
            ))])
        );
        assert_eq!(
            parse_input("(1\n"),
            Err(vec![ParseErrorKind::UnexpectedToken(TokenKind::Eof)])
        );
        assert_eq!(
            parse_input("1 2\n"),
            Err(vec![ParseErrorKind::UnexpectedToken(TokenKind::Integer(2))])
        );
    }

    #[test]
    fn parse_errors_preserve_the_offending_token_span() {
        let (tokens, _) = lex::lex(7, "1 2\n");
        let errors = parse(&tokens.expect("valid lexer input")).expect_err("parse fails");

        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0].value,
            ParseErrorKind::UnexpectedToken(TokenKind::Integer(2))
        );
        assert_eq!(errors[0].span.file, 7);
        assert_eq!((errors[0].span.start, errors[0].span.end), (2, 3));
    }

    #[test]
    fn parses_an_empty_token_stream() {
        assert_eq!(parse_input(""), Ok(Vec::new()));
    }
}
