use super::state_machine::{Nfa, StateId};

pub enum RegexComponent {
    Literal(Vec<u8>),
    Alternation(Vec<RegexComponent>, Vec<RegexComponent>),
    Group(Vec<RegexComponent>),
    Repeat {
        item: Box<RegexComponent>,
        min: usize,
        max: Option<usize>,
    },
}

pub struct RegexPattern {
    pattern: Vec<RegexComponent>,
}

impl RegexPattern {
    pub fn new(pattern: Vec<RegexComponent>) -> Self {
        Self { pattern }
    }

    pub fn to_nfa(&self) -> Nfa {
        let mut nfa = Nfa::new();
        let start_state = nfa.start_state;
        let end_state = nfa.add_state(true);

        self.build_nfa_sequence(&mut nfa, &self.pattern, start_state, end_state);
        nfa
    }

    fn build_nfa_sequence(
        &self,
        nfa: &mut Nfa,
        components: &[RegexComponent],
        start: StateId,
        end: StateId,
    ) {
        if components.is_empty() {
            nfa.add_transition(start, end, None);
            return;
        }

        let mut current_start = start;
        for (i, component) in components.iter().enumerate() {
            let current_end = if i == components.len() - 1 {
                end
            } else {
                nfa.add_state(false)
            };

            self.build_nfa_component(nfa, component, current_start, current_end);
            current_start = current_end;
        }
    }

    fn build_nfa_component(
        &self,
        nfa: &mut Nfa,
        component: &RegexComponent,
        start: StateId,
        end: StateId,
    ) {
        match component {
            RegexComponent::Literal(bytes) => {
                let mut current = start;
                for (i, &byte) in bytes.iter().enumerate() {
                    let next = if i == bytes.len() - 1 {
                        end
                    } else {
                        nfa.add_state(false)
                    };
                    nfa.add_transition(current, next, Some(byte));
                    current = next;
                }
            }

            RegexComponent::Alternation(left, right) => {
                let left_start = nfa.add_state(false);
                let left_end = nfa.add_state(false);
                let right_start = nfa.add_state(false);
                let right_end = nfa.add_state(false);

                nfa.add_transition(start, left_start, None);
                nfa.add_transition(start, right_start, None);

                self.build_nfa_sequence(nfa, left, left_start, left_end);
                self.build_nfa_sequence(nfa, right, right_start, right_end);

                nfa.add_transition(left_end, end, None);
                nfa.add_transition(right_end, end, None);
            }

            RegexComponent::Group(components) => {
                self.build_nfa_sequence(nfa, components, start, end);
            }

            RegexComponent::Repeat { item, min, max } => match (min, max) {
                (0, None) => {
                    nfa.add_transition(start, end, None);
                    let loop_start = nfa.add_state(false);
                    let loop_end = nfa.add_state(false);
                    nfa.add_transition(start, loop_start, None);
                    self.build_nfa_component(nfa, item, loop_start, loop_end);
                    nfa.add_transition(loop_end, loop_start, None);
                    nfa.add_transition(loop_end, end, None);
                }
                (1, None) => {
                    let loop_start = nfa.add_state(false);
                    let loop_end = nfa.add_state(false);
                    self.build_nfa_component(nfa, item, start, loop_start);
                    nfa.add_transition(loop_start, loop_end, None);
                    self.build_nfa_component(nfa, item, loop_end, loop_end);
                    nfa.add_transition(loop_end, end, None);
                }
                (0, Some(1)) => {
                    nfa.add_transition(start, end, None);
                    self.build_nfa_component(nfa, item, start, end);
                }
                _ => {
                    let mut current = start;
                    for i in 0..*min {
                        let next = if i == min - 1 && *max == Some(*min) {
                            end
                        } else {
                            nfa.add_state(false)
                        };
                        self.build_nfa_component(nfa, item, current, next);
                        current = next;
                    }

                    if let Some(max_val) = max {
                        for i in *min..*max_val {
                            let next = if i == max_val - 1 {
                                end
                            } else {
                                nfa.add_state(false)
                            };
                            nfa.add_transition(current, end, None);
                            self.build_nfa_component(nfa, item, current, next);
                            current = next;
                        }
                    }
                }
            },
        }
    }
}
