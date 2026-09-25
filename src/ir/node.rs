use std::marker::PhantomData;

use crate::ir::{Control, ControlId, Value, ValueId};
use crate::util::arena::ArenaId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Constant {
    Number(i64),
    String(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    Start,
    Return,

    Constant(Constant),
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
}
