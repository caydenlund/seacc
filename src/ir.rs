use std::marker::PhantomData;

use crate::util::arena::{Arena, ArenaId};

mod node;
pub use node::*;

macro_rules! node {
    (Return($control:expr)) => {
        $crate::ir::node::NodeSpec::<()>::return_($control, None)
    };
    (Return($control:expr, $value:expr)) => {
        $crate::ir::node::NodeSpec::<()>::return_($control, Some($value))
    };
    (Constant($value:expr)) => {
        $crate::ir::node::NodeSpec::<()>::constant($value)
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Control {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Value {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dead {}

trait NodeType: Clone + Copy + PartialEq + Eq {}
impl NodeType for Control {}
impl NodeType for Value {}
impl NodeType for Dead {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(private_bounds)]
pub struct NodeId<K: NodeType> {
    id: ArenaId<Node>,
    _kind: PhantomData<K>,
}

#[allow(private_bounds)]
impl<K: NodeType> NodeId<K> {
    const fn new(id: ArenaId<Node>) -> Self {
        Self {
            id,
            _kind: PhantomData,
        }
    }
}

pub type ControlId = NodeId<Control>;
pub type ValueId = NodeId<Value>;
pub type DeadId = NodeId<Dead>;

#[derive(Clone)]
pub struct Graph {
    nodes: Arena<Node>,
    start: ControlId,
}

impl Default for Graph {
    fn default() -> Self {
        let mut nodes = Arena::new();
        let start = ControlId::new(nodes.insert(Node {
            kind: NodeKind::Start,
            inputs: vec![],
            outputs: vec![],
        }));
        Self { nodes, start }
    }
}

impl Graph {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    #[allow(private_bounds)]
    pub fn add<K: NodeType>(&mut self, spec: NodeSpec<K>) -> NodeId<K> {
        let id = self.nodes.insert(Node {
            kind: spec.kind,
            inputs: spec.inputs.clone(),
            outputs: vec![],
        });
        for input in spec.inputs {
            self.nodes.get_mut(input).unwrap().outputs.push(id);
        }
        NodeId::new(id)
    }

    #[must_use]
    #[allow(private_bounds)]
    pub fn get<K: NodeType>(&self, id: NodeId<K>) -> Option<&Node> {
        self.nodes.get(id.id)
    }

    #[must_use]
    #[allow(private_bounds)]
    pub fn get_mut<K: NodeType>(&mut self, id: NodeId<K>) -> Option<&mut Node> {
        self.nodes.get_mut(id.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use NodeKind::*;
    use node::Constant::*;

    fn node(kind: NodeKind, inputs: &[ArenaId<Node>], outputs: &[ArenaId<Node>]) -> Node {
        Node {
            kind,
            inputs: inputs.to_owned(),
            outputs: outputs.to_owned(),
        }
    }

    #[test]
    fn ret() {
        let mut graph = Graph::new();
        let start = graph.start;
        assert_eq!(graph.get(start), Some(&node(Start, &[], &[])));

        let ret = graph.add(node![Return(start)]);
        assert_eq!(graph.get(start), Some(&node(Start, &[], &[ret.id])));
        assert_eq!(graph.get(ret), Some(&node(Return, &[start.id], &[])));
    }

    #[test]
    fn ret_const() {
        let mut graph = Graph::new();
        let start = graph.start;

        let constant = graph.add(node![Constant(Number(123))]);
        assert_eq!(
            graph.get(constant),
            Some(&node(Constant(Number(123)), &[], &[]))
        );

        let ret = graph.add(node![Return(start, constant)]);
        assert_eq!(
            graph.get(constant),
            Some(&node(Constant(Number(123)), &[], &[ret.id]))
        );
        assert_eq!(
            graph.get(ret),
            Some(&node(Return, &[start.id, constant.id], &[]))
        );
    }
}
