use std::{error::Error, fmt::Display};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexError {
    InvalidNumber { literal: String },
    InvalidStringEscape { escape: char },
    UnterminatedString,
}

impl Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidNumber { literal } => write!(f, "invalid number: '{literal}'"),
            Self::InvalidStringEscape { escape } => {
                write!(f, "invalid string escape: '\\{escape}'")
            }
            Self::UnterminatedString => write!(f, "unterminated string"),
        }
    }
}

impl Error for LexError {}

pub type LexResult<T> = Result<T, LexError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    Lex { source: LexError },
    EmptyInput,
    ExtraInput,
    UnclosedList,
    ExtraRparen,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lex { source } => write!(f, "lex error: {source}"),
            Self::EmptyInput => write!(f, "empty input"),
            Self::ExtraInput => write!(f, "extra input"),
            Self::UnclosedList => write!(f, "unclosed list"),
            Self::ExtraRparen => write!(f, "extra ')'"),
        }
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Self::Lex { source } = self {
            Some(source)
        } else {
            None
        }
    }
}

impl From<LexError> for ParseError {
    fn from(err: LexError) -> Self {
        Self::Lex { source: err }
    }
}

pub type ParseResult<T> = Result<T, ParseError>;
