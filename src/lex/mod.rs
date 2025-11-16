//! lex: Datatypes and routines for tokenizing an input program

mod error;
pub use error::LexError;

mod lexer;
pub use lexer::Lexer;

mod regex;
pub use regex::{RegexComponent, RegexPattern};

mod state_machine;
pub use state_machine::{Dfa, Nfa};

mod token;
pub use token::{Token, TokenType};
