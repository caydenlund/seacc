mod expr;
pub use expr::{Expr, ExprArena, ExprId};

mod parser;
use parser::Parser;

#[derive(Debug, Clone)]
pub struct ParseResult {
    pub exprs: ExprArena,
    pub root_expr: ExprId,
}

/// Parses the given text into a string
///
/// # Errors
/// When input is invalid
pub fn parse(s: &str) -> Result<ParseResult, String> {
    Parser::new(s).parse()
}

#[cfg(test)]
mod tests {
    use super::{Expr, parse};

    fn root(src: &str) -> (super::ParseResult, super::ExprId) {
        let result = parse(src).unwrap();
        let id = result.root_expr;
        (result, id)
    }

    #[test]
    fn parse_int() {
        let (result, id) = root("  12345  ");

        assert!(matches!(result.exprs.get(id), Some(Expr::Integer(12_345))));
    }

    #[test]
    fn parse_empty_list() {
        let (result, id) = root("(  )");

        assert!(matches!(result.exprs.get(id), Some(Expr::List(items)) if items.is_empty()));
    }

    #[test]
    fn parse_nested_values() {
        let (result, id) = root("(1 (#t #\\x) 3)");
        let Expr::List(items) = result.exprs.get(id).unwrap() else {
            panic!("root should be a list");
        };

        assert_eq!(items.len(), 3);
        assert!(matches!(result.exprs.get(items[0]), Some(Expr::Integer(1))));
        let Expr::List(nested) = result.exprs.get(items[1]).unwrap() else {
            panic!("second item should be a list");
        };
        assert!(matches!(
            result.exprs.get(nested[0]),
            Some(Expr::Bool(true))
        ));
        assert!(matches!(result.exprs.get(nested[1]), Some(Expr::Char('x'))));
        assert!(matches!(result.exprs.get(items[2]), Some(Expr::Integer(3))));
    }

    #[test]
    fn parse_string_escapes() {
        let (result, id) = root(r#""line\n\t\\end""#);

        assert!(matches!(
            result.exprs.get(id),
            Some(Expr::String(value)) if value == "line\n\t\\end"
        ));
    }

    #[test]
    fn parse_hash_literals() {
        for (src, expected) in [
            ("#t", Expr::Bool(true)),
            ("#f", Expr::Bool(false)),
            ("#\\space", Expr::Char(' ')),
        ] {
            let (result, id) = root(src);
            match (result.exprs.get(id), expected) {
                (Some(Expr::Bool(actual)), Expr::Bool(expected)) => assert_eq!(*actual, expected),
                (Some(Expr::Char(actual)), Expr::Char(expected)) => assert_eq!(*actual, expected),
                _ => panic!("unexpected parsed literal"),
            }
        }
    }

    #[test]
    fn reject_bad_inputs() {
        for (src, expected) in [
            ("", "unexpected end of input"),
            (")", "unexpected ')'"),
            ("(1", "unterminated list"),
            (r#""bad\q""#, "invalid string escape sequence: '\\q'"),
            ("#x", "invalid char after hash: 'x'"),
        ] {
            assert_eq!(parse(src).unwrap_err(), expected);
        }
    }

    #[test]
    fn reject_trailing_text() {
        assert_eq!(
            parse("1 2").unwrap_err(),
            "unexpected input after expression: '2'"
        );
    }
}
