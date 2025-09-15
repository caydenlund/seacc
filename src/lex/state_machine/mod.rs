//! lex/state-machine: State machines implementing lexical token patterns

use std::collections::BTreeSet;

/// A unique ID for a state, corresponding to an index in a list
pub type StateId = usize;
/// A set of state IDs
pub type StateSet = BTreeSet<StateId>;

pub mod dfa;
pub use dfa::Dfa;

pub mod nfa;
pub use nfa::Nfa;
