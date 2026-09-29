use crate::util::arena::{Arena, ArenaId};

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
    pub outputs: Vec<NodeId>,
}

pub type NodeId = ArenaId<Node>;

#[derive(Clone)]
pub struct Graph {
    nodes: Arena<Node>,
    start: NodeId,
}

impl Default for Graph {
    fn default() -> Self {
        let mut nodes = Arena::new();
        let start = nodes.insert(Node {
            kind: NodeKind::Start,
            inputs: Vec::new(),
            outputs: Vec::new(),
        });
        Self { nodes, start }
    }
}

impl Graph {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub const fn start(&self) -> NodeId {
        self.start
    }

    #[must_use]
    pub fn add(&mut self, spec: NodeSpec) -> NodeId {
        match spec {
            NodeSpec::Return(ctrl, val) => self.add_node(NodeKind::Return, &[ctrl, val]),
            NodeSpec::Constant(constant) => self.add_node(NodeKind::Constant(constant), &[]),
            NodeSpec::Mul(lhs, rhs) => self.add_node(NodeKind::Mul, &[lhs, rhs]),
            NodeSpec::Div(lhs, rhs) => self.add_node(NodeKind::Div, &[lhs, rhs]),
            NodeSpec::Mod(lhs, rhs) => self.add_node(NodeKind::Mod, &[lhs, rhs]),
            NodeSpec::Add(lhs, rhs) => self.add_node(NodeKind::Add, &[lhs, rhs]),
            NodeSpec::Sub(lhs, rhs) => self.add_node(NodeKind::Sub, &[lhs, rhs]),
        }
    }

    #[must_use]
    #[inline]
    fn add_node(&mut self, kind: NodeKind, inputs: &[NodeId]) -> NodeId {
        let id = self.nodes.insert(Node {
            kind,
            inputs: inputs.to_vec(),
            outputs: Vec::new(),
        });
        for &input in inputs {
            self.nodes.get_mut(input).unwrap().outputs.push(id);
        }
        id
    }

    #[must_use]
    #[allow(private_bounds)]
    pub fn get(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id)
    }

    #[must_use]
    #[allow(private_bounds)]
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id)
    }
}
