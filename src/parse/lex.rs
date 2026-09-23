use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexError {
    InvalidNumber(String),
    InvalidStringEscape(char),
    UnterminatedString,
}

pub type LexResult<T> = Result<T, LexError>;

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
        self.next_if(|next| next == kind)
    }

    pub fn peek(&mut self) -> LexResult<Option<&Token>> {
        if self.next.is_none() {
            self.next = self.next()?;
        }
        Ok(self.next.as_ref())
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
            Some(ch) => Err(LexError::InvalidStringEscape(ch)),
            None => Err(LexError::UnterminatedString),
        }
    }

    fn lex_number(&mut self) -> LexResult<TokenKind> {
        let value = self.take_atom();
        value
            .parse::<i64>()
            .map(TokenKind::Number)
            .map_err(|_| LexError::InvalidNumber(value))
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
    fn bad_literal() {
        assert_eq!(kinds("12abc"), Err(LexError::InvalidNumber("12abc".into())));
        assert_eq!(kinds("\"unterminated"), Err(LexError::UnterminatedString));
        assert_eq!(kinds("\"\\q\""), Err(LexError::InvalidStringEscape('q')));
    }
}
