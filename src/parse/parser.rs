use crate::ir::{Constant, Node, NodeArena, NodeId, NodeKind};
use crate::parse::lex::TokenKind;
use crate::parse::{Ast, Lexer, ParseError, ParseResult};

#[derive(Clone)]
pub struct Parser<'s> {
    tokens: Lexer<'s>,
    nodes: NodeArena,
    start: NodeId,
}

impl<'src> Parser<'src> {
    pub fn new(src: &'src str) -> Self {
        let mut nodes = NodeArena::new();
        let start = nodes.insert(Node::new(NodeKind::Start));

        Self {
            tokens: Lexer::new(src),
            nodes,
            start,
        }
    }

    pub fn parse(mut self) -> ParseResult<Ast> {
        let expr = self.parse_expr()?;
        if let Some(tok) = self.tokens.next()? {
            return Err(ParseError::ExtraInput(tok));
        }
        let ret = self.insert_node(NodeKind::Return {
            control: self.start,
            value: expr,
        });
        self.add_output(expr, ret);

        Ok(Ast {
            nodes: self.nodes,
            start: self.start,
        })
    }

    fn parse_expr(&mut self) -> ParseResult<NodeId> {
        match self.tokens.peek()?.map(|tok| &tok.kind) {
            Some(TokenKind::Number(n)) => {
                let n = *n;
                self.tokens.next()?;
                Ok(self.insert_constant(Constant::Number(n)))
            }
            None => todo!(),
            _ => todo!(),
        }
    }

    fn insert_constant(&mut self, value: Constant) -> NodeId {
        let id = self.insert_node(NodeKind::Constant {
            control: self.start,
            value,
        });
        self.add_output(self.start, id);
        id
    }

    fn add_output(&mut self, sup: NodeId, sub: NodeId) {
        self.nodes.get_mut(sup).unwrap().outputs.push(sub);
    }

    fn insert_node(&mut self, kind: NodeKind) -> NodeId {
        self.nodes.insert(Node::new(kind))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::parse::Token;

    use super::*;

    fn walk_ast(input: &str) -> ParseResult<Vec<NodeKind>> {
        let ast = crate::parse::parse(input)?;
        let mut walk = vec![ast.start];
        let mut seen = HashSet::from([ast.start]);
        let mut idx = 0;
        while idx < walk.len() {
            for &output in &ast.nodes.get(walk[idx]).unwrap().outputs {
                if seen.insert(output) {
                    walk.push(output);
                }
            }
            idx += 1;
        }

        Ok(walk
            .into_iter()
            .map(|id| ast.nodes.get(id).unwrap().kind.clone())
            .collect())
    }

    #[test]
    fn parse_int() {
        assert!(matches!(
            walk_ast("  123 ").as_deref(),
            Ok([
                NodeKind::Start,
                NodeKind::Constant {
                    control: _,
                    value: Constant::Number(123)
                },
                NodeKind::Return {
                    control: _,
                    value: _
                }
            ])
        ));
        assert!(matches!(
            walk_ast("1 2"),
            Err(ParseError::ExtraInput(Token {
                kind: TokenKind::Number(2)
            }))
        ));
    }
}
