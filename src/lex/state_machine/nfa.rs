use super::{Dfa, StateId, StateSet};
use std::collections::HashMap;

pub type NfaTransition = Option<u8>;

#[derive(Default, Debug, Clone)]
pub struct NfaState {
    pub is_accepting: bool,
    pub transitions: Vec<(NfaTransition, StateId)>,
}

#[derive(Debug, Clone)]
pub struct Nfa {
    pub states: Vec<NfaState>,
    pub start_state: StateId,
}

impl Nfa {
    pub fn new() -> Self {
        Self {
            states: vec![NfaState::default()],
            start_state: 0,
        }
    }

    pub fn add_state(&mut self, is_accepting: bool) -> StateId {
        let state = NfaState {
            is_accepting,
            transitions: Vec::new(),
        };

        self.states.push(state);
        self.states.len() - 1
    }

    pub fn add_transition(&mut self, from: StateId, to: StateId, transition: NfaTransition) {
        if let Some(state) = self.states.get_mut(from) {
            state.transitions.push((transition, to));
        }
    }

    pub fn epsilon_closure(&self, states: &StateSet) -> StateSet {
        let mut closure = states.clone();
        let mut stack: Vec<StateId> = states.iter().cloned().collect();

        while let Some(state_id) = stack.pop() {
            if let Some(state) = self.states.get(state_id) {
                for (transition, next_state) in &state.transitions {
                    if transition.is_none() && !closure.contains(next_state) {
                        closure.insert(*next_state);
                        stack.push(*next_state);
                    }
                }
            }
        }

        closure
    }

    pub fn transition(&self, states: &StateSet, input: u8) -> StateSet {
        let mut next_set = StateSet::new();

        for state_id in self.epsilon_closure(states) {
            if let Some(state) = self.states.get(state_id) {
                for (transition, next_state) in &state.transitions {
                    if let Some(symbol) = transition {
                        if *symbol == input {
                            next_set.insert(*next_state);
                        }
                    }
                }
            }
        }

        next_set
    }

    pub fn to_dfa(&self) -> Dfa {
        let mut dfa = Dfa::new();
        let mut state_map: HashMap<StateSet, StateId> = HashMap::new();
        let mut worklist: Vec<StateSet> = Vec::new();

        let start_closure: StateSet = self.epsilon_closure(&StateSet::from([self.start_state]));
        let start_is_accepting = start_closure
            .iter()
            .any(|&id| self.states.get(id).is_some_and(|s| s.is_accepting));

        dfa.states[0].is_accepting = start_is_accepting;
        state_map.insert(start_closure.clone(), 0);
        worklist.push(start_closure);

        while let Some(current_set) = worklist.pop() {
            let current_dfa_state = *state_map.get(&current_set).unwrap();

            for input in 0..=255u8 {
                let next_set = self.transition(&current_set, input);

                if !next_set.is_empty() {
                    let next_closure: StateSet = self.epsilon_closure(&next_set);

                    let next_dfa_state = if let Some(&existing_state) = state_map.get(&next_closure)
                    {
                        existing_state
                    } else {
                        let is_accepting = next_closure
                            .iter()
                            .any(|&id| self.states.get(id).is_some_and(|s| s.is_accepting));

                        let new_state = dfa.add_state(is_accepting);
                        state_map.insert(next_closure.clone(), new_state);
                        worklist.push(next_closure.clone());
                        new_state
                    };

                    dfa.add_transition(current_dfa_state, next_dfa_state, input);
                }
            }
        }

        dfa
    }

    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph NFA {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=circle];\n");

        for (state_id, state) in self.states.iter().enumerate() {
            if state.is_accepting {
                dot.push_str(&format!("  {state_id} [shape=doublecircle];\n"));
            }
        }

        dot.push_str("  start [shape=point, style=invis];\n");
        dot.push_str(&format!("  start -> {};\n", self.start_state));

        for (from_state, state) in self.states.iter().enumerate() {
            for (transition, to_state) in &state.transitions {
                let label = match transition {
                    Some(byte) => {
                        if byte.is_ascii_graphic() && *byte != b'"' && *byte != b'\\' {
                            format!("\"{}\"", *byte as char)
                        } else {
                            format!("\"\\\\x{byte:02x}\"")
                        }
                    }
                    None => "\"ε\"".to_string(),
                };
                dot.push_str(&format!("  {from_state} -> {to_state} [label={label}];\n"));
            }
        }

        dot.push_str("}\n");
        dot
    }
}

impl Default for Nfa {
    fn default() -> Self {
        Self::new()
    }
}
