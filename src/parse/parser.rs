use crate::parse::lex::TokenKind;
use crate::parse::{Ast, AstNode, Lexer, ParseError, ParseResult};
use crate::util::arena::Arena;

#[derive(Clone)]
pub struct Parser<'s> {
    tokens: Lexer<'s>,
    nodes: Arena<AstNode>,
}

impl<'src> Parser<'src> {
    pub fn new(src: &'src str) -> Self {
        Self {
            tokens: Lexer::new(src),
            nodes: Arena::new(),
        }
    }

    pub fn parse(mut self) -> ParseResult<Ast> {
        let mut list_stack = Vec::new();
        let mut curr_list = Vec::new();

        while let Some(tok) = self.tokens.next()? {
            match tok.kind {
                TokenKind::Symbol(s) => curr_list.push(self.nodes.insert(AstNode::Symbol(s))),
                TokenKind::Number(n) => curr_list.push(self.nodes.insert(AstNode::Number(n))),
                TokenKind::String(s) => curr_list.push(self.nodes.insert(AstNode::String(s))),
                TokenKind::Lparen => {
                    list_stack.push(curr_list);
                    curr_list = Vec::new();
                }
                TokenKind::Rparen => {
                    let list = self.nodes.insert(AstNode::List(curr_list));
                    let Some(l) = list_stack.pop() else {
                        return Err(ParseError::ExtraRparen);
                    };
                    curr_list = l;
                    curr_list.push(list);
                }
            }
        }

        if !list_stack.is_empty() {
            return Err(ParseError::UnclosedList);
        }
        if curr_list.is_empty() {
            return Err(ParseError::EmptyInput);
        }
        if curr_list.len() > 1 {
            return Err(ParseError::ExtraInput);
        }

        let root = curr_list[0];
        Ok(Ast {
            nodes: self.nodes,
            root,
        })
    }
}
