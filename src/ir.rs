use crate::util::arena::Arena;

mod error;
pub use error::{GraphError, GraphResult};

mod node;
pub use node::{Constant, Node, NodeId, NodeKind, NodeSpec};

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
        for (idx, &input) in inputs.iter().enumerate() {
            self.nodes.get_mut(input).unwrap().outputs.push((idx, id));
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

    pub fn replace_uses(&mut self, old_id: NodeId, new_id: NodeId) -> GraphResult<()> {
        let err = |id| GraphError::MissingNode { id };
        let old = self.nodes.get_mut(old_id).ok_or_else(|| err(old_id))?;
        let outputs: Vec<_> = old.outputs.drain(0..).collect();

        let new = self.nodes.get_mut(new_id).ok_or_else(|| err(new_id))?;
        new.outputs.extend(&outputs);

        for (idx, id) in outputs {
            self.get_mut(id).ok_or_else(|| err(id))?.inputs[idx] = new_id;
        }

        Ok(())
    }

    pub fn kill_if_unused(&mut self, id: NodeId) -> GraphResult<()> {
        let node = self.nodes.get(id).ok_or(GraphError::MissingNode { id })?;

        if node.outputs.is_empty() {
            self.nodes.remove(id);
        }

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[must_use]
    pub fn serialize(graph: &Graph) -> GraphResult<String> {
        fn serialize_rec(graph: &Graph, id: NodeId) -> GraphResult<String> {
            use NodeKind::*;
            let Node { kind, inputs, .. } = graph.get(id).ok_or(GraphError::MissingNode { id })?;
            Ok(match kind {
                Start => unreachable!(),
                Return => serialize_rec(graph, inputs[1])?,
                Constant(super::Constant::Number(n)) => n.to_string(),
                Mul => format!(
                    "({} * {})",
                    serialize_rec(graph, inputs[0])?,
                    serialize_rec(graph, inputs[1])?
                ),
                Div => format!(
                    "({} / {})",
                    serialize_rec(graph, inputs[0])?,
                    serialize_rec(graph, inputs[1])?
                ),
                Mod => todo!(),
                Add => format!(
                    "({} + {})",
                    serialize_rec(graph, inputs[0])?,
                    serialize_rec(graph, inputs[1])?
                ),
                Sub => format!(
                    "({} - {})",
                    serialize_rec(graph, inputs[0])?,
                    serialize_rec(graph, inputs[1])?
                ),
            })
        }

        let start = graph.get(graph.start()).unwrap();
        serialize_rec(graph, start.outputs[0].1)
    }
}
