mod lexer;

mod regex;
pub use regex::{RegexComponent, RegexPattern};

mod state_machine;
pub use state_machine::{Dfa, Nfa};

mod token;
pub use token::TokenType;
