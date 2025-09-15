//! lex/regex: Regular expressions for defining lexical token patterns

use super::TokenType;
use super::state_machine::{Nfa, StateId};
use crate::util::BitSet;

/// A single component of a regular expression
#[derive(Debug, Clone)]
pub enum RegexComponent {
    /// A literal string (as a sequence of bytes)
    Literal(Vec<u8>),
    /// Matches either one of two variants
    Alternation(Vec<RegexComponent>, Vec<RegexComponent>),
    /// A series of [`RegexComponent`]s
    Group(Vec<RegexComponent>),
    /// A set of allowed characters
    CharSet(BitSet<u32, 8>),
    /// Repetition of a [`RegexComponent`]
    Repeat {
        /// The item to repeat
        item: Box<RegexComponent>,
        /// The minimum amount of times to match
        min: usize,
        /// The (optional) maximum amount of times to match
        max: Option<usize>,
    },
}

/// A regular expression as a series of [`RegexComponent`]
#[derive(Debug, Clone)]
pub struct RegexPattern {
    /// The series of [`RegexComponent`]s to match
    pattern: Vec<RegexComponent>,
}

impl RegexComponent {
    /// Returns a new [`RegexComponent::CharSet`] for the given range of characters
    #[must_use]
    pub fn char_range(start: u8, end: u8) -> Self {
        let mut char_set = BitSet::new();
        for c in start..=end {
            char_set.set(c as usize);
        }
        Self::CharSet(char_set)
    }

    /// Returns a new [`RegexComponent::CharSet`] for the given series of characters
    #[must_use]
    pub fn chars(chars: &[u8]) -> Self {
        let mut char_set = BitSet::new();
        for &c in chars {
            char_set.set(c as usize);
        }
        Self::CharSet(char_set)
    }
}

impl RegexPattern {
    /// Returns a new [`RegexPattern`] for the given series of [`RegexComponent`]s
    #[must_use]
    pub fn new(pattern: &[RegexComponent]) -> Self {
        Self {
            pattern: pattern.to_vec(),
        }
    }

    /// Generates a [`Nfa`] for this regular expression
    #[must_use]
    pub fn to_nfa(&self, token_type: TokenType) -> Nfa {
        let mut nfa = Nfa::new();
        let start_state = nfa.start_state;
        let end_state = nfa.add_state(Some(token_type));

        self.build_nfa_sequence(&mut nfa, &self.pattern, start_state, end_state);
        nfa
    }

    /// Recursively generates a [`Nfa`] for a series of [`RegexComponent`]
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
                nfa.add_state(None)
            };

            self.build_nfa_component(nfa, component, current_start, current_end);
            current_start = current_end;
        }
    }

    /// Recursively generates a [`Nfa`] for a single [`RegexComponent`]
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
                        nfa.add_state(None)
                    };
                    nfa.add_transition(current, next, Some(byte));
                    current = next;
                }
            }

            RegexComponent::Alternation(left, right) => {
                let left_start = nfa.add_state(None);
                let left_end = nfa.add_state(None);
                let right_start = nfa.add_state(None);
                let right_end = nfa.add_state(None);

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

            RegexComponent::CharSet(char_set) => {
                for byte in char_set.iter() {
                    #[allow(clippy::cast_possible_truncation)]
                    nfa.add_transition(start, end, Some(byte as u8));
                }
            }

            RegexComponent::Repeat { item, min, max } => match (min, max) {
                (0, None) => {
                    nfa.add_transition(start, end, None);
                    let loop_start = nfa.add_state(None);
                    let loop_end = nfa.add_state(None);
                    nfa.add_transition(start, loop_start, None);
                    self.build_nfa_component(nfa, item, loop_start, loop_end);
                    nfa.add_transition(loop_end, loop_start, None);
                    nfa.add_transition(loop_end, end, None);
                }
                (1, None) => {
                    let loop_start = nfa.add_state(None);
                    let loop_end = nfa.add_state(None);
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
                            nfa.add_state(None)
                        };
                        self.build_nfa_component(nfa, item, current, next);
                        current = next;
                    }

                    if let Some(max_val) = max {
                        for i in *min..*max_val {
                            let next = if i == max_val - 1 {
                                end
                            } else {
                                nfa.add_state(None)
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
