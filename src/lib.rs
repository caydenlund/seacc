#![warn(
    clippy::all,
    clippy::cargo,
    clippy::nursery,
    clippy::pedantic,
    // missing_docs,
    rustdoc::all
)]

pub mod lex;
pub mod parse;

mod intern;
pub use intern::{StringIntern, StringRef};
mod span;
pub use span::{FileId, LineMap, Span, Spanned};
