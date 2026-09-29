use crate::util::arena::ArenaId;

#[derive(Debug, Clone)]
pub enum Constant {
    Number(i64),
}

#[derive(Debug, Clone)]
pub enum NodeKind {
    Start,
    Return,

    Constant(Constant),
    Mul,
    Div,
    Mod,
    Add,
    Sub,
}

#[derive(Debug, Clone)]
pub enum NodeSpec {
    Return(NodeId, NodeId),

    Constant(Constant),
    Mul(NodeId, NodeId),
    Div(NodeId, NodeId),
    Mod(NodeId, NodeId),
    Add(NodeId, NodeId),
    Sub(NodeId, NodeId),
}

#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub inputs: Vec<NodeId>,
    pub outputs: Vec<(usize, NodeId)>,
}

pub type NodeId = ArenaId<Node>;
