use crate::Span;

mod punct;
pub use punct::Punct;

mod keyword;
pub use keyword::Keyword;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Punct(Punct),
    Keyword(Keyword),
    Ident(String),
    Integer(u64),
    Decimal(f64),
    String(String),
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}
