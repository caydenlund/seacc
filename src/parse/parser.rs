use crate::ir::{Constant, ControlId, Graph, ValueId};
use crate::node;
use crate::parse::lex::TokenKind;
use crate::parse::{Lexer, ParseError, ParseResult, Token};

#[derive(Clone)]
pub struct Parser<'s> {
    tokens: Lexer<'s>,
    graph: Graph,
    start: ControlId,
}

impl<'src> Parser<'src> {
    pub fn new(src: &'src str) -> Self {
        let graph = Graph::default();
        let start = graph.start();
        Self {
            tokens: Lexer::new(src),
            graph,
            start,
        }
    }

    pub fn parse(mut self) -> ParseResult<Graph> {
        let value = self.parse_expr()?;
        let _ret = self.graph.add(node![Return(self.start, value)]);

        if let Some(tok) = self.tokens.next()? {
            return Err(ParseError::ExtraInput(tok));
        }

        Ok(self.graph)
    }

    fn parse_expr(&mut self) -> ParseResult<ValueId> {
        match self.tokens.peek()? {
            Some(&TokenKind::Number(n)) => {
                let _ = self.tokens.next();
                Ok(self.graph.add(node![Constant(Constant::Number(n))]))
            }
            Some(&TokenKind::Lparen) => self.parse_list(),
            _ => todo!(),
        }
    }

    fn parse_list(&mut self) -> ParseResult<ValueId> {
        let lparen = self.tokens.next()?;
        debug_assert_eq!(
            lparen,
            Some(Token {
                kind: TokenKind::Lparen
            })
        );

        let Some(op_tok) = self.tokens.next()? else {
            return Err(ParseError::UnclosedList);
        };
        let TokenKind::Symbol(op) = op_tok.kind else {
            todo!("not a symbol: {:?}", op_tok.kind);
        };

        let mut contents = Vec::new();

        loop {
            let Some(kind) = self.tokens.peek()? else {
                return Err(ParseError::UnclosedList);
            };
            if kind == &TokenKind::Rparen {
                self.tokens.next()?;
                break;
            }
            contents.push(self.parse_expr()?);
        }

        match (&op as &str, contents.len()) {
            ("*", 0) => Ok(self.graph.add(node![Constant(Constant::Number(1))])),
            ("*" | "+", 1) => Ok(contents[0]),
            ("*", 2..) => {
                let mut acc = self.graph.add(node![Mul(contents[0], contents[1])]);
                for expr in contents.into_iter().skip(2) {
                    acc = self.graph.add(node![Mul(acc, expr)]);
                }
                Ok(acc)
            }
            ("/", 0) => Err(ParseError::WrongArity(op, 0)),
            ("/", 1) => todo!("invert"),
            ("/", 2..) => {
                let mut acc = self.graph.add(node![Div(contents[0], contents[1])]);
                for expr in contents.into_iter().skip(2) {
                    acc = self.graph.add(node![Div(acc, expr)]);
                }
                Ok(acc)
            }
            ("+", 0) => Ok(self.graph.add(node![Constant(Constant::Number(0))])),
            ("+", 2..) => {
                let mut acc = self.graph.add(node![Add(contents[0], contents[1])]);
                for expr in contents.into_iter().skip(2) {
                    acc = self.graph.add(node![Add(acc, expr)]);
                }
                Ok(acc)
            }
            ("-", 1) => {
                let zero = self.graph.add(node![Constant(Constant::Number(0))]);
                Ok(self.graph.add(node![Sub(zero, contents[0])]))
            }
            ("-", 2..) => {
                let mut acc = self.graph.add(node![Sub(contents[0], contents[1])]);
                for expr in contents.into_iter().skip(2) {
                    acc = self.graph.add(node![Sub(acc, expr)]);
                }
                Ok(acc)
            }
            _ => todo!("unhandled op: {op}"),
        }
    }
}
