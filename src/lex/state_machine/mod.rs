use std::collections::BTreeSet;

pub type StateId = usize;
pub type StateSet = BTreeSet<StateId>;

pub mod nfa;
pub use nfa::Nfa;

pub mod dfa;
pub use dfa::Dfa;
