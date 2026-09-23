use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexError {}

pub type LexResult<T> = Result<T, LexError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Identifier(String),
    Boolean(bool),
    Number(i64),
    Char(char),
    String(String),
    Lparen,
    Rparen,
    OpenVec,
    Dot,
    Quot,
    Unquot, // TODO: what is this?
    At,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
}

pub struct Lexer<'s> {
    chars: Peekable<Chars<'s>>,
    next: Option<Token>,
}

impl<'s> Lexer<'s> {
    pub fn new(src: &'s str) -> Self {
        Self {
            chars: src.chars().peekable(),
            next: None,
        }
    }

    pub fn next(&mut self) -> LexResult<Option<Token>> {
        if self.next.is_some() {
            return Ok(self.next.take());
        }
        if self.cpeek().is_none() {
            return Ok(None);
        }

        macro_rules! eat_yield {
            ($kind:expr) => {{
                self.cnext();
                Ok(Some(Token { kind: $kind }))
            }};
        }

        self.skip_ws();
        match self.cpeek() {
            Some(';') => todo!("comment"),
            Some('#') => {
                self.cnext();
                match self.cpeek() {
                    Some('(') => eat_yield!(TokenKind::OpenVec),
                    Some('t') => eat_yield!(TokenKind::Boolean(true)),
                    Some('f') => eat_yield!(TokenKind::Boolean(false)),
                    Some('\\') => todo!("character"),
                    _ => todo!(),
                }
            }
            Some('"') => todo!("string"),
            Some('(') => eat_yield!(TokenKind::Lparen),
            Some(')') => eat_yield!(TokenKind::Rparen),
            Some('.') => todo!("start of number or TokenKind::Dot"),
            Some('0'..='9') => todo!("number"),
            Some('-' | '+') => todo!("number or symbol"),
            Some('\'' | '`') => todo!(),
            Some('@') => eat_yield!(TokenKind::At),
            Some(_) => todo!(),
            None => Ok(None),
        }
    }

    pub fn next_if<F: Fn(&TokenKind) -> bool>(&mut self, f: F) -> LexResult<Option<Token>> {
        if let Some(next) = self.next()? {
            if f(&next.kind) {
                Ok(Some(next))
            } else {
                self.next = Some(next);
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub fn next_if_eq(&mut self, kind: &TokenKind) -> LexResult<Option<Token>> {
        if let Some(next) = self.next()? {
            if &next.kind == kind {
                Ok(Some(next))
            } else {
                self.next = Some(next);
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn skip_ws(&mut self) {
        while self.cnext_if(|c| c.is_whitespace()).is_some() {}
    }

    fn cpeek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn cnext(&mut self) -> Option<char> {
        self.chars.next()
    }

    fn cnext_if<F: Fn(char) -> bool>(&mut self, f: F) -> Option<char> {
        if self.cpeek().is_some_and(f) {
            self.cnext()
        } else {
            None
        }
    }

    fn cnext_if_eq(&mut self, ch: char) -> Option<char> {
        if self.cpeek().is_some_and(|c| c == ch) {
            self.cnext()
        } else {
            None
        }
    }
}
