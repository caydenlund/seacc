use std::iter::Peekable;
use std::str::Chars;

use crate::parse::{LexError, LexResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Symbol(String),
    Number(i64),
    String(String),
    Lparen,
    Rparen,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
}

#[derive(Clone)]
pub struct Lexer<'s> {
    chars: Peekable<Chars<'s>>,
    cnext: Option<char>,
    next: Option<Token>,
}

impl<'s> Lexer<'s> {
    pub fn new(src: &'s str) -> Self {
        Self {
            chars: src.chars().peekable(),
            cnext: None,
            next: None,
        }
    }

    pub fn next(&mut self) -> LexResult<Option<Token>> {
        if self.next.is_some() {
            return Ok(self.next.take());
        }
        self.skip_ws_comment();
        let Some(ch) = self.cpeek() else {
            return Ok(None);
        };
        let kind = match ch {
            '"' => self.lex_string()?,
            '(' => {
                self.cnext();
                TokenKind::Lparen
            }
            ')' => {
                self.cnext();
                TokenKind::Rparen
            }
            '0'..='9' => self.lex_number()?,
            '+' | '-' if self.cpeek2().is_some_and(|next| next.is_ascii_digit()) => {
                self.lex_number()?
            }
            _ => TokenKind::Symbol(self.take_atom()),
        };
        Ok(Some(Token { kind }))
    }

    fn lex_string(&mut self) -> LexResult<TokenKind> {
        let lquote = self.cnext();
        debug_assert_eq!(lquote, Some('"'));
        let mut value = String::new();
        loop {
            match self.cnext() {
                Some('"') => return Ok(TokenKind::String(value)),
                Some('\\') => value.push(self.lex_string_escape()?),
                Some(ch) => value.push(ch),
                None => return Err(LexError::UnterminatedString),
            }
        }
    }

    fn lex_string_escape(&mut self) -> LexResult<char> {
        match self.cnext() {
            Some('t') => Ok('\t'),
            Some('n') => Ok('\n'),
            Some('"') => Ok('"'),
            Some('\\') => Ok('\\'),
            Some(escape) => Err(LexError::InvalidStringEscape { escape }),
            None => Err(LexError::UnterminatedString),
        }
    }

    fn lex_number(&mut self) -> LexResult<TokenKind> {
        let value = self.take_atom();
        value
            .parse::<i64>()
            .map(TokenKind::Number)
            .map_err(|_| LexError::InvalidNumber { literal: value })
    }

    fn take_atom(&mut self) -> String {
        let mut value = String::new();
        while self.cpeek().is_some_and(|ch| !Self::is_delimiter(Some(ch))) {
            value.push(self.cnext().expect("peeked character"));
        }
        value
    }

    fn skip_ws_comment(&mut self) {
        loop {
            while self.cnext_if(char::is_whitespace).is_some() {}
            if self.cnext_if_eq(';').is_none() {
                break;
            }
            while self.cnext().is_some_and(|ch| ch != '\n') {}
        }
    }

    fn is_delimiter(ch: Option<char>) -> bool {
        ch.is_none_or(|ch| ch.is_whitespace() || matches!(ch, '(' | ')' | '"' | ';' | '\''))
    }

    fn cpeek(&mut self) -> Option<char> {
        if self.cnext.is_none() {
            self.cnext = self.chars.next();
        }
        self.cnext
    }

    fn cpeek2(&mut self) -> Option<char> {
        if self.cnext.is_none() {
            self.cnext = self.chars.next();
        }
        self.chars.peek().copied()
    }

    fn cnext(&mut self) -> Option<char> {
        if self.cnext.is_some() {
            self.cnext.take()
        } else {
            self.chars.next()
        }
    }

    fn cnext_if<F: Fn(char) -> bool>(&mut self, f: F) -> Option<char> {
        if self.cpeek().is_some_and(f) {
            self.cnext()
        } else {
            None
        }
    }

    fn cnext_if_eq(&mut self, ch: char) -> Option<char> {
        if self.cpeek() == Some(ch) {
            self.cnext()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Result<Vec<TokenKind>, LexError> {
        let mut lexer = Lexer::new(src);
        let mut result = Vec::new();
        while let Some(token) = lexer.next()? {
            result.push(token.kind);
        }
        Ok(result)
    }

    #[test]
    fn comments() {
        assert_eq!(kinds(""), Ok(vec![]));
        assert_eq!(kinds("; foo"), Ok(vec![]));
        assert_eq!(kinds("; foo\n123\n; bar"), Ok(vec![TokenKind::Number(123)]));
    }

    #[test]
    fn list() {
        use TokenKind::{Lparen, Rparen, Symbol};
        let sym = |s| Symbol(String::from(s));

        assert_eq!(
            kinds("a (b (c) d) e"),
            Ok(vec![
                sym("a"),
                Lparen,
                sym("b"),
                Lparen,
                sym("c"),
                Rparen,
                sym("d"),
                Rparen,
                sym("e")
            ])
        );
    }

    #[test]
    fn bad_literal() {
        assert_eq!(
            kinds("12abc"),
            Err(LexError::InvalidNumber {
                literal: "12abc".into()
            })
        );
        assert_eq!(kinds("\"unterminated"), Err(LexError::UnterminatedString));
        assert_eq!(
            kinds("\"\\q\""),
            Err(LexError::InvalidStringEscape { escape: 'q' })
        );
    }
}
