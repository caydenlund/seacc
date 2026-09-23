use crate::util::arena::{Arena, ArenaId};

pub type NodeId = ArenaId<Node>;
pub type NodeArena = Arena<Node>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Constant {
    Boolean(bool),
    Number(i64),
    Character(char),
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
    kind: NodeKind,
    outputs: Vec<NodeId>,
}
