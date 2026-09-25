use std::marker::PhantomData;

use crate::ir::{Control, ControlId, Value, ValueId};
use crate::util::arena::ArenaId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Constant {
    Number(i64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    Start,
    Return,

    Constant(Constant),
    Mul,
    Div,
    Mod,
    Rem,
    Add,
    Sub,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub kind: NodeKind,
    pub inputs: Vec<ArenaId<Self>>,
    pub outputs: Vec<ArenaId<Self>>,
}

pub struct NodeSpec<K> {
    pub kind: NodeKind,
    pub inputs: Vec<ArenaId<Node>>,
    _type: PhantomData<K>,
}

impl<K> NodeSpec<K> {
    #[must_use]
    pub fn return_(control: ControlId, value: Option<ValueId>) -> NodeSpec<Control> {
        let inputs = value.map_or_else(|| vec![control.id], |value| vec![control.id, value.id]);

        NodeSpec {
            kind: NodeKind::Return,
            inputs,
            _type: PhantomData,
        }
    }

    #[must_use]
    pub const fn constant(value: Constant) -> NodeSpec<Value> {
        NodeSpec {
            kind: NodeKind::Constant(value),
            inputs: Vec::new(),
            _type: PhantomData,
        }
    }

    #[must_use]
    pub fn multiply(lhs: ValueId, rhs: ValueId) -> NodeSpec<Value> {
        NodeSpec {
            kind: NodeKind::Mul,
            inputs: vec![lhs.id, rhs.id],
            _type: PhantomData,
        }
    }

    #[must_use]
    pub fn divide(lhs: ValueId, rhs: ValueId) -> NodeSpec<Value> {
        NodeSpec {
            kind: NodeKind::Div,
            inputs: vec![lhs.id, rhs.id],
            _type: PhantomData,
        }
    }

    #[must_use]
    pub fn modulo(lhs: ValueId, rhs: ValueId) -> NodeSpec<Value> {
        NodeSpec {
            kind: NodeKind::Mod,
            inputs: vec![lhs.id, rhs.id],
            _type: PhantomData,
        }
    }

    #[must_use]
    pub fn remainder(lhs: ValueId, rhs: ValueId) -> NodeSpec<Value> {
        NodeSpec {
            kind: NodeKind::Rem,
            inputs: vec![lhs.id, rhs.id],
            _type: PhantomData,
        }
    }

    #[must_use]
    pub fn add(lhs: ValueId, rhs: ValueId) -> NodeSpec<Value> {
        NodeSpec {
            kind: NodeKind::Add,
            inputs: vec![lhs.id, rhs.id],
            _type: PhantomData,
        }
    }

    #[must_use]
    pub fn subtract(lhs: ValueId, rhs: ValueId) -> NodeSpec<Value> {
        NodeSpec {
            kind: NodeKind::Sub,
            inputs: vec![lhs.id, rhs.id],
            _type: PhantomData,
        }
    }
}
