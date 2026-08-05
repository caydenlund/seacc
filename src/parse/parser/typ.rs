use super::Parser;
use crate::{Spanned, lex::token::TokenKind, parse::ast::Type};

impl Parser<'_> {
    pub(super) fn try_parse_type(&mut self) -> Option<Spanned<Type>> {
        // TODO: every type except `int` and `float`
        match self.peek().map(|t| &t.value) {
            Some(TokenKind::Ident(s)) => {
                let typ = match s as &str {
                    "int" => Type::Int,
                    "float" => Type::Float,
                    _ => return None,
                };
                Some(Spanned {
                    value: typ,
                    span: self.next().unwrap().span,
                })
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Span, Spanned, lex::token::Token};

    use super::*;

    fn lex(input: &str) -> Vec<Token> {
        crate::lex::lex(7, &format!("{input}\n"))
            .0
            .unwrap_or_else(|e| panic!("failed to tokenize input '{input}': {e:?}"))
    }

    #[test]
    fn try_parse_type() {
        let tokens = lex("int x float");
        let mut parser = Parser::new(&tokens);
        // `int`
        assert_eq!(
            parser.try_parse_type(),
            Some(Spanned {
                value: Type::Int,
                span: Span {
                    file: 7,
                    start: 0,
                    end: 3
                }
            })
        );
        // `x`
        assert_eq!(parser.try_parse_type(), None);
        assert_eq!(
            parser.next(),
            Some(&Spanned {
                value: TokenKind::Ident("x".into()),
                span: Span {
                    file: 7,
                    start: 4,
                    end: 5
                }
            })
        );
        // `float`
        assert_eq!(
            parser.try_parse_type(),
            Some(Spanned {
                value: Type::Float,
                span: Span {
                    file: 7,
                    start: 6,
                    end: 11
                }
            })
        );
    }
}
