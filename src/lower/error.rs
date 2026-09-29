use std::{error::Error, fmt::Display};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LowerError {
    InvalidInvocation,
    UnboundSymbol(String),
    WrongArity,
}

impl Display for LowerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Error for LowerError {}
