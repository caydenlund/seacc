#![warn(
    clippy::all,
    clippy::cargo,
    clippy::nursery,
    clippy::pedantic,
    // missing_docs,
    rustdoc::all
)]

pub mod lex;

mod span;
pub use span::{FileId, LineMap, Span, Spanned};
