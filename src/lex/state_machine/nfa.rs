use super::{Dfa, StateId, StateSet};
use crate::lex::TokenType;
use std::collections::HashMap;

pub type NfaTransition = Option<u8>;

#[derive(Default, Debug, Clone)]
pub struct NfaState {
    pub token_type: Option<TokenType>,
    pub transitions: Vec<(NfaTransition, StateId)>,
}

impl NfaState {
    pub fn is_accepting(&self) -> bool {
        self.token_type.is_some()
    }
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

    pub fn add_state(&mut self, token_type: Option<TokenType>) -> StateId {
        let state = NfaState {
            token_type,
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

    pub fn merge(nfas: &[Nfa]) -> Nfa {
        if nfas.is_empty() {
            return Nfa::new();
        }

        if nfas.len() == 1 {
            return nfas[0].clone();
        }

        let mut merged = Nfa::new();
        let mut old_to_new_mappings = Vec::new();

        // Copy all states from each NFA, keeping track of state ID mappings
        for nfa in nfas {
            let mut mapping = HashMap::new();

            for (old_id, state) in nfa.states.iter().enumerate() {
                let new_id = merged.add_state(state.token_type.clone());
                mapping.insert(old_id, new_id);
            }

            old_to_new_mappings.push(mapping);
        }

        // Copy all transitions using the new state IDs
        for (nfa_idx, nfa) in nfas.iter().enumerate() {
            let mapping = &old_to_new_mappings[nfa_idx];

            for (old_from, state) in nfa.states.iter().enumerate() {
                let new_from = mapping[&old_from];

                for (transition, old_to) in &state.transitions {
                    let new_to = mapping[old_to];
                    merged.add_transition(new_from, new_to, *transition);
                }
            }
        }

        // Add epsilon transitions from the new start state to each NFA's start state
        for (nfa_idx, nfa) in nfas.iter().enumerate() {
            let mapping = &old_to_new_mappings[nfa_idx];
            let new_start = mapping[&nfa.start_state];
            merged.add_transition(merged.start_state, new_start, None);
        }

        merged
    }

    pub fn to_dfa(&self) -> Dfa {
        let mut dfa = Dfa::new();
        let mut state_map: HashMap<StateSet, StateId> = HashMap::new();
        let mut worklist: Vec<StateSet> = Vec::new();

        let start_closure: StateSet = self.epsilon_closure(&StateSet::from([self.start_state]));
        // For the start state, we don't have a token type (it's non-accepting)
        // The start state will have token_type = None
        dfa.states[0].token_type = None;
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
                        // Find the token type for this closure - prioritize the first accepting state we find
                        let token_type = next_closure
                            .iter()
                            .find_map(|&id| self.states.get(id)?.token_type.clone());

                        let new_state = dfa.add_state(token_type);
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
            if state.is_accepting() {
                let token_label = state
                    .token_type
                    .as_ref()
                    .map(|t| format!("{t:?}"))
                    .unwrap_or("Unknown".into())
                    .replace('"', "\\\"");
                dot.push_str(&format!(
                    "  {state_id} [shape=doublecircle, label=\"{token_label}\"];\n"
                ));
            }
        }

        dot.push_str("  start [shape=point, style=invis];\n");
        dot.push_str(&format!("  start -> {};\n", self.start_state));

        // Group transitions by (from_state, to_state) pairs
        let mut transitions: HashMap<(usize, usize), Vec<Option<u8>>> = HashMap::new();

        for (from_state, state) in self.states.iter().enumerate() {
            for (transition, to_state) in &state.transitions {
                transitions
                    .entry((from_state, *to_state))
                    .or_default()
                    .push(*transition);
            }
        }

        // Generate consolidated edges
        for ((from_state, to_state), inputs) in transitions {
            let label = self.format_nfa_transition_label(&inputs);
            dot.push_str(&format!(
                "  {from_state} -> {to_state} [label=\"{label}\"];\n"
            ));
        }

        dot.push_str("}\n");
        dot
    }

    fn format_nfa_transition_label(&self, inputs: &[Option<u8>]) -> String {
        if inputs.len() == 1 {
            match inputs[0] {
                Some(byte) => {
                    if byte.is_ascii_graphic() && byte != b'"' && byte != b'\\' {
                        (byte as char).to_string()
                    } else {
                        format!("\\\\x{byte:02x}")
                    }
                }
                None => "ε".to_string(),
            }
        } else {
            // Separate epsilon transitions from character transitions
            let mut epsilons = Vec::new();
            let mut chars = Vec::new();

            for &input in inputs {
                match input {
                    Some(byte) => chars.push(byte),
                    None => epsilons.push(()),
                }
            }

            let mut parts = Vec::new();

            if !epsilons.is_empty() {
                if epsilons.len() == 1 {
                    parts.push("ε".to_string());
                } else {
                    parts.push(format!("ε({})", epsilons.len()));
                }
            }

            if !chars.is_empty() {
                // Group consecutive ranges and individual characters for regular chars
                chars.sort_unstable();

                let mut result = String::new();
                let mut i = 0;

                while i < chars.len() {
                    let start = chars[i];
                    let mut end = start;

                    // Find consecutive range
                    while i + 1 < chars.len() && chars[i + 1] == end + 1 {
                        i += 1;
                        end = chars[i];
                    }

                    if !result.is_empty() {
                        result.push(',');
                    }

                    if start == end {
                        // Single character
                        if start.is_ascii_graphic() && start != b'"' && start != b'\\' {
                            result.push(start as char);
                        } else {
                            result.push_str(&format!("\\\\x{start:02x}"));
                        }
                    } else if end == start + 1 {
                        // Two consecutive characters, show individually
                        if start.is_ascii_graphic() && start != b'"' && start != b'\\' {
                            result.push(start as char);
                        } else {
                            result.push_str(&format!("\\\\x{start:02x}"));
                        }
                        result.push(',');
                        if end.is_ascii_graphic() && end != b'"' && end != b'\\' {
                            result.push(end as char);
                        } else {
                            result.push_str(&format!("\\\\x{end:02x}"));
                        }
                    } else {
                        // Range of 3 or more characters
                        if start.is_ascii_graphic()
                            && start != b'"'
                            && start != b'\\'
                            && end.is_ascii_graphic()
                            && end != b'"'
                            && end != b'\\'
                        {
                            result.push_str(&format!("{}-{}", start as char, end as char));
                        } else {
                            result.push_str(&format!("\\\\x{start:02x}-\\\\x{end:02x}"));
                        }
                    }

                    i += 1;
                }

                parts.push(result);
            }

            parts.join(",")
        }
    }
}

impl Default for Nfa {
    fn default() -> Self {
        Self::new()
    }
}
