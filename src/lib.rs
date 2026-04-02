//! seacc: A sea-of-nodes C compiler, written in Rust

#![warn(
    clippy::all,
    clippy::cargo,
    // clippy::missing_docs_in_private_items,
    clippy::nursery,
    clippy::pedantic,
    // missing_docs,
    rustdoc::all
)]

pub mod pp_lexer;
pub mod preprocessor;
pub mod source_reader;
pub mod span;
pub mod token;
