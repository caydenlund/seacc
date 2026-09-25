use std::marker::PhantomData;

use crate::util::arena::{Arena, ArenaId};

mod node;
pub use node::*;

#[macro_export]
macro_rules! node {
    (Return($control:expr)) => {
        $crate::ir::NodeSpec::<()>::return_($control, None)
    };
    (Return($control:expr, $value:expr)) => {
        $crate::ir::NodeSpec::<()>::return_($control, Some($value))
    };
    (Constant($value:expr)) => {
        $crate::ir::NodeSpec::<()>::constant($value)
    };
    (Mul($lhs:expr, $rhs:expr)) => {
        $crate::ir::NodeSpec::<()>::multiply($lhs, $rhs)
    };
    (Div($lhs:expr, $rhs:expr)) => {
        $crate::ir::NodeSpec::<()>::divide($lhs, $rhs)
    };
    (Mod($lhs:expr, $rhs:expr)) => {
        $crate::ir::NodeSpec::<()>::modulo($lhs, $rhs)
    };
    (Rem($lhs:expr, $rhs:expr)) => {
        $crate::ir::NodeSpec::<()>::remainder($lhs, $rhs)
    };
    (Add($lhs:expr, $rhs:expr)) => {
        $crate::ir::NodeSpec::<()>::add($lhs, $rhs)
    };
    (Sub($lhs:expr, $rhs:expr)) => {
        $crate::ir::NodeSpec::<()>::subtract($lhs, $rhs)
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

    /// Adds the given node specification to the graph
    ///
    /// # Panics
    /// Never
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

    #[must_use]
    pub const fn start(&self) -> ControlId {
        self.start
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

    fn serialize_from(graph: &Graph, node: ArenaId<Node>) -> String {
        let node = graph.nodes.get(node).expect("missing node");
        match &node.kind {
            Start => "Start".into(),
            Return => match node.inputs.len() {
                1 => format!("Return(ctrl={})", serialize_from(graph, node.inputs[0])),
                2 => format!(
                    "Return(ctrl={}, val={})",
                    serialize_from(graph, node.inputs[0]),
                    serialize_from(graph, node.inputs[1])
                ),
                _ => panic!("invalid number of return inputs"),
            },
            Constant(node::Constant::Number(n)) => format!("#{n}"),
            Mul => format!(
                "({} * {})",
                serialize_from(graph, node.inputs[0]),
                serialize_from(graph, node.inputs[1])
            ),
            Div => format!(
                "({} / {})",
                serialize_from(graph, node.inputs[0]),
                serialize_from(graph, node.inputs[1])
            ),
            Mod => format!(
                "({} mod {})",
                serialize_from(graph, node.inputs[0]),
                serialize_from(graph, node.inputs[1])
            ),
            Rem => format!(
                "({} rem {})",
                serialize_from(graph, node.inputs[0]),
                serialize_from(graph, node.inputs[1])
            ),
            Add => format!(
                "({} + {})",
                serialize_from(graph, node.inputs[0]),
                serialize_from(graph, node.inputs[1])
            ),
            Sub => format!(
                "({} - {})",
                serialize_from(graph, node.inputs[0]),
                serialize_from(graph, node.inputs[1])
            ),
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
            serialize_from(&graph, ret.id),
            "Return(ctrl=Start, val=#123)"
        );
        assert_eq!(
            graph.get(constant),
            Some(&node(Constant(Number(123)), &[], &[ret.id]))
        );
        assert_eq!(
            graph.get(ret),
            Some(&node(Return, &[start.id, constant.id], &[]))
        );
    }

    #[test]
    fn arith() {
        let mut graph = Graph::new();
        let start = graph.start;

        let mut lhs = graph.add(node![Constant(Number(2))]);
        let mut rhs = graph.add(node![Constant(Number(3))]);
        lhs = graph.add(node![Mul(lhs, rhs)]);
        rhs = graph.add(node![Constant(Number(4))]);
        rhs = graph.add(node![Div(lhs, rhs)]);
        lhs = graph.add(node![Constant(Number(1))]);
        rhs = graph.add(node![Add(lhs, rhs)]);

        let ret = graph.add(node![Return(start, rhs)]);
        assert_eq!(
            serialize_from(&graph, ret.id),
            "Return(ctrl=Start, val=(#1 + ((#2 * #3) / #4)))"
        );
    }
}
