use std::{error::Error, fmt::Display};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LowerError {
    EmptyInvocation,
    NonSymbolCallee {
        callee: String,
    },
    UnboundSymbol {
        symbol: String,
    },
    WrongArity {
        operator: String,
        minimum: usize,
        actual: usize,
    },
}

impl Display for LowerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyInvocation => write!(f, "invalid invocation: empty list"),
            Self::NonSymbolCallee { callee } => {
                write!(f, "invalid invocation: '{callee}' is not a symbol")
            }
            Self::UnboundSymbol { symbol } => write!(f, "unbound symbol: '{symbol}'"),
            Self::WrongArity {
                operator,
                minimum: 1,
                actual,
            } => write!(
                f,
                "wrong arity for '{operator}': expected at least 1 argument, got {actual}"
            ),
            Self::WrongArity {
                operator,
                minimum,
                actual,
            } => write!(
                f,
                "wrong arity for '{operator}': expected at least {minimum} arguments, got {actual}"
            ),
        }
    }
}

impl Error for LowerError {}

pub type LowerResult<T> = Result<T, LowerError>;
