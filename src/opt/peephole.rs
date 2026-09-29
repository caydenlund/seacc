use crate::ir::{Constant, Graph, Node, NodeKind, NodeSpec};
use crate::{
    ir::NodeId,
    opt::{Pass, PassResult},
};

pub struct Peephole;

impl Pass for Peephole {
    fn name(&self) -> &'static str {
        "peephole"
    }

    fn run(&self, graph: &mut Graph) -> PassResult {
        fn peephole(graph: &mut Graph, id: NodeId) -> (NodeId, bool) {
            fn peephole_binop(
                graph: &mut Graph,
                id: NodeId,
                op: fn(i64, i64) -> i64,
            ) -> (NodeId, bool) {
                let node = graph.get(id).unwrap();
                debug_assert!(matches!(
                    node.kind,
                    NodeKind::Mul | NodeKind::Div | NodeKind::Mod | NodeKind::Add | NodeKind::Sub
                ));
                let (l_id, r_id) = (node.inputs[0], node.inputs[1]);
                let (l_id, l_changed) = peephole(graph, l_id);
                let (r_id, r_changed) = peephole(graph, r_id);
                if let (
                    Some(Node {
                        kind: NodeKind::Constant(Constant::Number(lhs)),
                        ..
                    }),
                    Some(Node {
                        kind: NodeKind::Constant(Constant::Number(rhs)),
                        ..
                    }),
                ) = (graph.get(l_id), graph.get(r_id))
                {
                    let new = graph.add(NodeSpec::Constant(Constant::Number(op(*lhs, *rhs))));
                    graph.replace_uses(id, new).expect("graph violation");
                    graph.kill_if_unused(l_id).expect("graph violation");
                    graph.kill_if_unused(r_id).expect("graph violation");
                    (new, true)
                } else {
                    (id, l_changed || r_changed)
                }
            }

            let node = graph.get(id).unwrap();
            match &node.kind {
                #[allow(clippy::unnecessary_fold)]
                NodeKind::Start => (
                    id,
                    node.outputs
                        .clone()
                        .into_iter()
                        .fold(false, |changed, o| peephole(graph, o.1).1 || changed),
                ),
                NodeKind::Return => peephole(graph, node.inputs[1]),
                NodeKind::Constant(_) => (id, false),
                NodeKind::Mul => peephole_binop(graph, id, |a, b| a * b),
                NodeKind::Div => peephole_binop(graph, id, |a, b| a / b),
                NodeKind::Mod => peephole_binop(graph, id, |a, b| a % b),
                NodeKind::Add => peephole_binop(graph, id, |a, b| a + b),
                NodeKind::Sub => peephole_binop(graph, id, |a, b| a - b),
            }
        }

        PassResult {
            changed: peephole(graph, graph.start()).1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ir::{GraphResult, tests::serialize},
        lower::{LowerResult, tests::lower},
    };

    #[allow(clippy::unnecessary_wraps)]
    fn ok<T>(s: &str) -> Result<String, T> {
        Ok(s.into())
    }

    fn peepholed(s: &str) -> LowerResult<Graph> {
        let mut g = lower(s)?;
        Peephole.run(&mut g);
        Ok(g)
    }

    fn peeps(s: &str) -> GraphResult<String> {
        serialize(&peepholed(s).expect("peep"))
    }

    #[test]
    fn const_combine() {
        assert_eq!(peeps("(+ 1)"), ok("1"));
        assert_eq!(peeps("(+ 1 2)"), ok("3"));
        assert_eq!(peeps("(+ 1 2 3)"), ok("6"));

        assert_eq!(peeps("(* 1)"), ok("1"));
        assert_eq!(peeps("(* 1 2)"), ok("2"));
        assert_eq!(peeps("(* 1 2 3)"), ok("6"));

        assert_eq!(peeps("(- 6)"), ok("-6"));
        assert_eq!(peeps("(/ 6 2)"), ok("3"));
        assert_eq!(peeps("(/ 8 2 2)"), ok("2"));

        assert_eq!(peeps("(- 3 2)"), ok("1"));
        assert_eq!(peeps("(- 3 2 1)"), ok("0"));

        assert_eq!(peeps("(/ (* 2 (+ 3 1)) 2)"), ok("4"));
    }
}
