use std::{error::Error, fmt::Display};

use crate::ir::NodeId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphError {
    MissingNode { id: NodeId },
}

impl Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingNode { id } => write!(f, "missing node: {id:?}"),
        }
    }
}

impl Error for GraphError {}

pub type GraphResult<T> = Result<T, GraphError>;
