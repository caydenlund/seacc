use crate::ir::{self, Graph, NodeId, NodeSpec};
use crate::parse::{Ast, AstNode, AstNodeId};
use crate::util::arena::Arena;

mod error;
pub use error::{LowerError, LowerResult};

/// Lowers the given [`Ast`] to the sea-of-nodes [`Graph`] representation
///
/// # Errors
/// If the abstract syntax tree has malformed Scheme constructs
pub fn lower(Ast { nodes, root }: &Ast) -> LowerResult<Graph> {
    struct Context {
        nodes: Arena<AstNode>,
        start: NodeId,
    }

    fn lower_rec(context: &Context, graph: &mut Graph, node: AstNodeId) -> LowerResult<NodeId> {
        match context.nodes.get(node) {
            Some(AstNode::Number(n)) => Ok(graph.add(NodeSpec::Constant(ir::Constant::Number(*n)))),
            Some(AstNode::String(_)) => todo!(),
            Some(AstNode::Symbol(symbol)) => Err(LowerError::UnboundSymbol {
                symbol: symbol.clone(),
            }),
            Some(AstNode::List(nodes)) => {
                let Some((sym, args)) = nodes.split_first() else {
                    return Err(LowerError::EmptyInvocation);
                };
                let sym = match context.nodes.get(*sym) {
                    Some(AstNode::Symbol(sym)) => sym,
                    Some(callee) => {
                        return Err(LowerError::NonSymbolCallee {
                            callee: format!("{callee:?}"),
                        });
                    }
                    None => unreachable!(),
                };
                match (sym as &str, args.len()) {
                    ("*", 0) => Ok(graph.add(NodeSpec::Constant(ir::Constant::Number(1)))),
                    ("*", 1..) => {
                        let lhs = lower_rec(context, graph, args[0])?;
                        args[1..].iter().try_fold(lhs, |lhs, node| {
                            let rhs = lower_rec(context, graph, *node)?;
                            Ok(graph.add(NodeSpec::Mul(lhs, rhs)))
                        })
                    }
                    //
                    ("/", actual) if actual < 1 => Err(LowerError::WrongArity {
                        operator: sym.clone(),
                        minimum: 1,
                        actual,
                    }),
                    ("/", 1) => todo!(),
                    ("/", 2..) => {
                        let lhs = lower_rec(context, graph, args[0])?;
                        args[1..].iter().try_fold(lhs, |lhs, node| {
                            let rhs = lower_rec(context, graph, *node)?;
                            Ok(graph.add(NodeSpec::Div(lhs, rhs)))
                        })
                    }
                    //
                    ("+", 0) => Ok(graph.add(NodeSpec::Constant(ir::Constant::Number(0)))),
                    #[allow(clippy::match_same_arms)]
                    ("+", 1..) => {
                        let lhs = lower_rec(context, graph, args[0])?;
                        args[1..].iter().try_fold(lhs, |lhs, node| {
                            let rhs = lower_rec(context, graph, *node)?;
                            Ok(graph.add(NodeSpec::Add(lhs, rhs)))
                        })
                    }
                    //
                    #[allow(clippy::match_same_arms)]
                    ("-", actual) if actual < 1 => Err(LowerError::WrongArity {
                        operator: sym.clone(),
                        minimum: 1,
                        actual,
                    }),
                    ("-", 1) => {
                        let lhs = graph.add(NodeSpec::Constant(ir::Constant::Number(0)));
                        let rhs = lower_rec(context, graph, args[0])?;
                        Ok(graph.add(NodeSpec::Sub(lhs, rhs)))
                    }
                    ("-", 2..) => {
                        let lhs = lower_rec(context, graph, args[0])?;
                        args[1..].iter().try_fold(lhs, |lhs, node| {
                            let rhs = lower_rec(context, graph, *node)?;
                            Ok(graph.add(NodeSpec::Sub(lhs, rhs)))
                        })
                    }
                    _ => Err(LowerError::UnboundSymbol {
                        symbol: sym.clone(),
                    }),
                }
            }
            None => unreachable!(),
        }
    }

    let mut graph = Graph::new();
    let context = Context {
        nodes: nodes.clone(),
        start: graph.start(),
    };

    let ret_val = lower_rec(&context, &mut graph, *root)?;
    let _ret = graph.add(NodeSpec::Return(context.start, ret_val));

    Ok(graph)
}

#[cfg(test)]
mod tests {
    use crate::ir::{Node, NodeKind};

    use super::*;

    fn lower(s: &str) -> LowerResult<Graph> {
        super::lower(&crate::parse::parse(s).unwrap())
    }

    fn serialize(graph: &Graph) -> String {
        fn serialize_rec(graph: &Graph, node: NodeId) -> String {
            use NodeKind::*;
            let Node { kind, inputs, .. } = graph.get(node).unwrap();
            match kind {
                Start | Return => unreachable!(),
                Constant(ir::Constant::Number(n)) => n.to_string(),
                Mul => format!(
                    "({} * {})",
                    serialize_rec(graph, inputs[0]),
                    serialize_rec(graph, inputs[1])
                ),
                Div => format!(
                    "({} / {})",
                    serialize_rec(graph, inputs[0]),
                    serialize_rec(graph, inputs[1])
                ),
                Mod => todo!(),
                Add => format!(
                    "({} + {})",
                    serialize_rec(graph, inputs[0]),
                    serialize_rec(graph, inputs[1])
                ),
                Sub => format!(
                    "({} - {})",
                    serialize_rec(graph, inputs[0]),
                    serialize_rec(graph, inputs[1])
                ),
            }
        }

        let start = graph.get(graph.start()).unwrap();
        let ret = graph.get(start.outputs[0]).unwrap();
        let val = ret.inputs[1];
        serialize_rec(graph, val)
    }

    fn lowstr(s: &str) -> LowerResult<String> {
        lower(s).map(|g| serialize(&g))
    }

    #[test]
    fn arith() {
        use LowerError::*;
        #[allow(clippy::unnecessary_wraps)]
        fn ok(s: &str) -> LowerResult<String> {
            Ok(s.to_string())
        }
        assert_eq!(lowstr("123"), ok("123"));

        assert_eq!(lowstr("(*)"), ok("1"));
        assert_eq!(lowstr("()"), Err(EmptyInvocation));
        assert_eq!(
            lowstr("(1)"),
            Err(NonSymbolCallee {
                callee: "Number(1)".into(),
            })
        );
        assert_eq!(
            lowstr("(/)"),
            Err(WrongArity {
                operator: "/".into(),
                minimum: 1,
                actual: 0,
            })
        );
        assert_eq!(lowstr("(+)"), ok("0"));
        assert_eq!(
            lowstr("(-)"),
            Err(WrongArity {
                operator: "-".into(),
                minimum: 1,
                actual: 0,
            })
        );

        assert_eq!(lowstr("(* 123)"), ok("123"));
        assert_eq!(lowstr("(+ 123)"), ok("123"));
        assert_eq!(lowstr("(- 123)"), ok("(0 - 123)"));
        assert_eq!(lowstr("(- -123)"), ok("(0 - -123)"));

        assert_eq!(lowstr("(* 1 2)"), ok("(1 * 2)"));
        assert_eq!(lowstr("(/ 1 2)"), ok("(1 / 2)"));
        assert_eq!(lowstr("(+ 1 2)"), ok("(1 + 2)"));
        assert_eq!(lowstr("(- 1 2)"), ok("(1 - 2)"));

        assert_eq!(lowstr("(* 1 2 3)"), ok("((1 * 2) * 3)"));
        assert_eq!(lowstr("(/ 1 2 3)"), ok("((1 / 2) / 3)"));
        assert_eq!(lowstr("(+ 1 2 3)"), ok("((1 + 2) + 3)"));
        assert_eq!(lowstr("(- 1 2 3)"), ok("((1 - 2) - 3)"));

        assert_eq!(lowstr("(* (- 1 2) 3)"), ok("((1 - 2) * 3)"));
        assert_eq!(lowstr("(/ (* 1 2) 3)"), ok("((1 * 2) / 3)"));
        assert_eq!(lowstr("(+ (/ 1 2) 3)"), ok("((1 / 2) + 3)"));
        assert_eq!(lowstr("(- (+ 1 2) 3)"), ok("((1 + 2) - 3)"));

        assert_eq!(lowstr("(* 1 (- 2 3))"), ok("(1 * (2 - 3))"));
        assert_eq!(lowstr("(/ 1 (* 2 3))"), ok("(1 / (2 * 3))"));
        assert_eq!(lowstr("(+ 1 (/ 2 3))"), ok("(1 + (2 / 3))"));
        assert_eq!(lowstr("(- 1 (+ 2 3))"), ok("(1 - (2 + 3))"));

        assert_eq!(
            lowstr("(- (+ 1 2 3) (* 1 (+ 2 3)))"),
            ok("(((1 + 2) + 3) - (1 * (2 + 3)))")
        );
    }
}
