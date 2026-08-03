use std::{error::Error, fmt::Display};

use crate::Spanned;
use crate::lex::token::TokenKind;

#[derive(Debug, Clone, PartialEq)]
pub enum ParseErrorKind {
    UnexpectedToken(TokenKind),
}

impl Display for ParseErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedToken(kind) => write!(f, "unexpected token: '{kind:?}'"),
        }
    }
}

impl Error for ParseErrorKind {}

pub type ParseError = Spanned<ParseErrorKind>;
