//! lex/state-machine/dfa: Deterministic Finite Automata for lexical token patterns

use super::StateId;
use crate::lex::TokenType;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

/// Transition on a single byte from one DFA state to another
pub type DfaTransition = u8;

/// A single DFA state, which may accept and generate a token and has associated state transitions
#[derive(Debug, Clone)]
pub struct DfaState {
    /// The token type that this state generates and accepts
    pub token_type: Option<TokenType>,
    /// Transitions on bytes to other DFA states
    pub transitions: [Option<StateId>; 256],
}

impl Default for DfaState {
    fn default() -> Self {
        Self {
            token_type: None,
            transitions: [None; 256],
        }
    }
}

/// A Deterministic Finite Automata
#[derive(Debug, Clone)]
pub struct Dfa {
    /// The contained states in this DFA
    pub states: Vec<DfaState>,
    /// The [`StateId`] of the first state
    pub start_state: StateId,
}

impl Dfa {
    /// Generates a default [`Dfa`] with a single (starting) state
    #[must_use]
    pub fn new() -> Self {
        Self {
            states: vec![DfaState::default()],
            start_state: 0,
        }
    }

    /// Adds a new state to the DFA
    ///
    /// If this state is accepting, it emits the given token type
    pub fn add_state(&mut self, token_type: Option<TokenType>) -> StateId {
        let state = DfaState {
            token_type,
            ..Default::default()
        };

        self.states.push(state);
        self.states.len() - 1
    }

    /// Adds a transition from one state to another on the given byte
    pub fn add_transition(&mut self, from: StateId, to: StateId, input: DfaTransition) {
        if let Some(state) = self.states.get_mut(from) {
            state.transitions[input as usize] = Some(to);
        }
    }

    /// Returns the transition from the given state on the given byte, if any
    #[must_use]
    pub fn transition(&self, from: StateId, input: DfaTransition) -> Option<StateId> {
        self.states
            .get(from)
            .and_then(|state| state.transitions[input as usize])
    }

    /// Returns a minimized DFA with an equivalent language
    #[must_use]
    pub fn minimize(&self) -> Self {
        if self.states.is_empty() {
            return Self::new();
        }

        // Step 1: Initial partition by token type (states with different token types cannot be merged)
        let mut token_type_groups: HashMap<Option<TokenType>, HashSet<StateId>> = HashMap::new();

        for (state_id, state) in self.states.iter().enumerate() {
            token_type_groups
                .entry(state.token_type.clone())
                .or_default()
                .insert(state_id);
        }

        let mut partitions: Vec<HashSet<StateId>> = token_type_groups.into_values().collect();

        // Step 2: Refine partitions until no more refinement is possible
        let mut changed = true;
        while changed {
            changed = false;
            let mut new_partitions = Vec::new();

            for partition in &partitions {
                let mut refined = self.refine_partition(partition, &partitions);
                if refined.len() > 1 {
                    changed = true;
                }
                new_partitions.append(&mut refined);
            }

            partitions = new_partitions;
        }

        // Step 3: Build minimized DFA
        self.build_minimized_dfa(&partitions)
    }

    /// Refines a partition by grouping states with identical transition signatures
    fn refine_partition(
        &self,
        partition: &HashSet<StateId>,
        all_partitions: &[HashSet<StateId>],
    ) -> Vec<HashSet<StateId>> {
        if partition.len() <= 1 {
            return vec![partition.clone()];
        }

        // Group states by their transition signatures
        let mut groups: HashMap<Vec<usize>, HashSet<StateId>> = HashMap::new();

        for &state_id in partition {
            let mut signature = Vec::new();

            // For each input symbol, find which partition the target state belongs to
            for input in 0..=255u8 {
                if let Some(target) = self.transition(state_id, input) {
                    // Find which partition contains the target state
                    let partition_idx = all_partitions
                        .iter()
                        .position(|p| p.contains(&target))
                        .unwrap_or(usize::MAX);
                    signature.push(partition_idx);
                } else {
                    signature.push(usize::MAX); // No transition
                }
            }

            groups.entry(signature).or_default().insert(state_id);
        }

        groups.into_values().collect()
    }

    /// Builds a minimized DFA from the refined partitions
    fn build_minimized_dfa(&self, partitions: &[HashSet<StateId>]) -> Self {
        let mut minimized = Self::new();
        let mut state_map: HashMap<StateId, StateId> = HashMap::new();

        // Find which partition contains the start state
        let start_partition = partitions
            .iter()
            .position(|p| p.contains(&self.start_state))
            .unwrap_or(0);

        // Create one state per partition
        for (partition_idx, partition) in partitions.iter().enumerate() {
            let representative = *partition.iter().next().unwrap();
            let token_type = self.states[representative].token_type.clone();

            let new_state_id = if partition_idx == start_partition {
                // Use state 0 for the partition containing start state
                minimized.states[0].token_type = token_type;
                0
            } else {
                minimized.add_state(token_type)
            };

            // Map all old states in this partition to the new state
            for &old_state in partition {
                state_map.insert(old_state, new_state_id);
            }
        }

        // Add transitions between partition representatives
        for partition in partitions {
            let representative = *partition.iter().next().unwrap();
            let from_state = state_map[&representative];

            for input in 0..=255u8 {
                if let Some(old_target) = self.transition(representative, input) {
                    let to_state = state_map[&old_target];
                    minimized.add_transition(from_state, to_state, input);
                }
            }
        }

        minimized
    }

    /// Builds a Graphviz DOT graph of this DFA
    #[must_use]
    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph DFA {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=circle];\n");

        for (state_id, state) in self.states.iter().enumerate() {
            if state.token_type.is_some() {
                let token_label = state
                    .token_type
                    .as_ref()
                    .map_or("Unknown".to_string(), |t| format!("{t:?}"))
                    .replace('"', "\\\"");
                let _ = writeln!(
                    dot,
                    "  {state_id} [shape=doublecircle, label=\"{token_label}\"];"
                );
            }
        }

        dot.push_str("  start [shape=point, style=invis];\n");
        let _ = writeln!(dot, "  start -> {};", self.start_state);

        // Group transitions by (from_state, to_state) pairs
        let mut transitions: HashMap<(usize, usize), Vec<u8>> = HashMap::new();

        for (from_state, state) in self.states.iter().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            for (input, to_state) in state
                .transitions
                .iter()
                .enumerate()
                .filter_map(|(input, to_state)| to_state.map(|to_state| (input as u8, to_state)))
            {
                transitions
                    .entry((from_state, to_state))
                    .or_default()
                    .push(input);
            }
        }

        // Generate consolidated edges
        for ((from_state, to_state), inputs) in transitions {
            let label = Self::format_transition_label(&inputs);
            let _ = writeln!(dot, "  {from_state} -> {to_state} [label=\"{label}\"];");
        }

        dot.push_str("}\n");
        dot
    }

    /// Formats a list of input bytes into a compact label for DOT graph edges
    fn format_transition_label(inputs: &[u8]) -> String {
        if inputs.len() == 1 {
            let input = inputs[0];
            if input.is_ascii_graphic() && input != b'"' && input != b'\\' {
                input as char
            } else {
                return format!("\\\\x{input:02x}");
            }
            .to_string()
        } else {
            // Group consecutive ranges and individual characters
            let mut sorted_inputs = inputs.to_vec();
            sorted_inputs.sort_unstable();

            let mut result = String::new();
            let mut i = 0;

            while i < sorted_inputs.len() {
                let start = sorted_inputs[i];
                let mut end = start;

                // Find consecutive range
                while i + 1 < sorted_inputs.len() && sorted_inputs[i + 1] == end + 1 {
                    i += 1;
                    end = sorted_inputs[i];
                }

                if !result.is_empty() {
                    result.push(',');
                }

                if start == end {
                    // Single character
                    if start.is_ascii_graphic() && start != b'"' && start != b'\\' {
                        result.push(start as char);
                    } else {
                        let _ = write!(result, "\\\\x{start:02x}");
                    }
                } else if end == start + 1 {
                    // Two consecutive characters, show individually
                    if start.is_ascii_graphic() && start != b'"' && start != b'\\' {
                        result.push(start as char);
                    } else {
                        let _ = write!(result, "\\\\x{start:02x}");
                    }
                    result.push(',');
                    if end.is_ascii_graphic() && end != b'"' && end != b'\\' {
                        result.push(end as char);
                    } else {
                        let _ = write!(result, "\\\\x{end:02x}");
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
                        let _ = write!(result, "{}-{}", start as char, end as char);
                    } else {
                        let _ = write!(result, "\\\\x{start:02x}-\\\\x{end:02x}");
                    }
                }

                i += 1;
            }

            result
        }
    }
}

impl Default for Dfa {
    fn default() -> Self {
        Self::new()
    }
}
