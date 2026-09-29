use crate::ir::Graph;

pub mod peephole;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct PassResult {
    pub changed: bool,
}

pub trait Pass {
    fn name(&self) -> &'static str;
    fn run(&self, graph: &mut Graph) -> PassResult;
}
