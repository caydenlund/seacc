use crate::util::arena::{Arena, ArenaId};

pub type NodeId = ArenaId<Node>;
pub type NodeArena = Arena<Node>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Constant {
    Number(i64),
    String(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    Start,
    Return { control: NodeId, value: NodeId },
    Constant { control: NodeId, value: Constant },
}

impl NodeKind {
    pub fn inputs(&self) -> Vec<NodeId> {
        match self {
            NodeKind::Start => Vec::new(),
            NodeKind::Return { control, value } => vec![*control, *value],
            NodeKind::Constant { control, value: _ } => vec![*control],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub kind: NodeKind,
    pub outputs: Vec<NodeId>,
}

impl Node {
    pub fn new(kind: NodeKind) -> Self {
        Self {
            kind,
            outputs: Vec::new(),
        }
    }
}
