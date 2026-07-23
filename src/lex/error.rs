use crate::Spanned;
use std::{error::Error, fmt::Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexErrorKind {
    UnexpectedChar(char),
    UnterminatedString,
}

impl Display for LexErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedChar(c) => write!(f, "unexpected character: '{c}'"),
            Self::UnterminatedString => write!(f, "unterminated string literal"),
        }
    }
}

impl Error for LexErrorKind {}

pub type LexError = Spanned<LexErrorKind>;
