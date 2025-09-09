use super::StateId;

pub type DfaTransition = u8;

#[derive(Debug, Clone)]
pub struct DfaState {
    pub is_accepting: bool,
    pub transitions: [Option<StateId>; 256],
}

impl Default for DfaState {
    fn default() -> Self {
        Self {
            is_accepting: false,
            transitions: [None; 256],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Dfa {
    pub states: Vec<DfaState>,
    pub start_state: StateId,
}

impl Dfa {
    pub fn new() -> Self {
        Self {
            states: vec![DfaState::default()],
            start_state: 0,
        }
    }

    pub fn add_state(&mut self, is_accepting: bool) -> StateId {
        let state = DfaState {
            is_accepting,
            ..Default::default()
        };

        self.states.push(state);
        self.states.len() - 1
    }

    pub fn add_transition(&mut self, from: StateId, to: StateId, input: DfaTransition) {
        if let Some(state) = self.states.get_mut(from) {
            state.transitions[input as usize] = Some(to);
        }
    }

    pub fn transition(&self, from: StateId, input: DfaTransition) -> Option<StateId> {
        self.states
            .get(from)
            .and_then(|state| state.transitions[input as usize])
    }

    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph DFA {\n");
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
            for (input, to_state) in state
                .transitions
                .iter()
                .enumerate()
                .filter_map(|(input, to_state)| to_state.map(|to_state| (input as u8, to_state)))
            {
                let label = if input.is_ascii_graphic() && input != b'"' && input != b'\\' {
                    format!("\"{}\"", input as char)
                } else {
                    format!("\"\\\\x{input:02x}\"")
                };
                dot.push_str(&format!("  {from_state} -> {to_state} [label={label}];\n"));
            }
        }

        dot.push_str("}\n");
        dot
    }
}

impl Default for Dfa {
    fn default() -> Self {
        Self::new()
    }
}
