mod error;
pub use error::PpLexerError;

use crate::source_reader::SourceReader;
use crate::span::{Span, Spanned};
use crate::token::{Identifier, PreprocessingToken, Punctuator};
use std::collections::VecDeque;

#[cfg(test)]
mod tests;

/// Preprocessing lexer
///
/// Implements translation phase 3
pub struct PpLexer<'src> {
    source: SourceReader<'src>,
    /// Lookahead buffer for peeking ahead
    lookahead: VecDeque<Spanned<'src, char>>,
    /// Pending token to return next
    pending: Option<Spanned<'src, PreprocessingToken>>,
}

impl<'src> PpLexer<'src> {
    /// Creates a new lexer from a source reader
    #[must_use]
    pub const fn new(source: SourceReader<'src>) -> Self {
        Self {
            source,
            lookahead: VecDeque::new(),
            pending: None,
        }
    }

    /// Peek at the next character without consuming it
    fn peek_char(&mut self) -> Result<Option<&Spanned<'src, char>>, PpLexerError<'src>> {
        if self.lookahead.is_empty()
            && let Some(result) = self.source.next()
        {
            self.lookahead.push_back(result?);
        }
        Ok(self.lookahead.front())
    }

    /// Peek at the nth character ahead (0-indexed)
    fn peek_n(&mut self, n: usize) -> Result<Option<&Spanned<'src, char>>, PpLexerError<'src>> {
        while self.lookahead.len() <= n {
            if let Some(result) = self.source.next() {
                self.lookahead.push_back(result?);
            } else {
                break;
            }
        }
        Ok(self.lookahead.get(n))
    }

    /// Read and consume the next character
    fn read_char(&mut self) -> Result<Option<Spanned<'src, char>>, PpLexerError<'src>> {
        if let Some(ch) = self.lookahead.pop_front() {
            return Ok(Some(ch));
        }
        match self.source.next() {
            Some(result) => Ok(Some(result?)),
            None => Ok(None),
        }
    }

    /// Read and consume the next character with known value, returning only the span
    #[inline]
    fn read_known_char(
        &mut self,
        expected_val: char,
    ) -> Result<Option<Span<'src>>, PpLexerError<'src>> {
        let Some(Spanned { value, span }) = self.read_char()? else {
            return Ok(None);
        };
        assert_eq!(value, expected_val);
        Ok(Some(span))
    }

    /// Check whether a character is a valid identifier start
    const fn is_identifier_start(ch: char) -> bool {
        ch.is_ascii_alphabetic() || ch == '_'
    }

    /// Check whether a character is a valid identifier continuation
    const fn is_identifier_continue(ch: char) -> bool {
        ch.is_ascii_alphanumeric() || ch == '_'
    }

    /// Check whether a character can start a pp-number
    const fn is_pp_number_start(ch: char) -> bool {
        ch.is_ascii_digit()
    }

    /// Check whether a character is whitespace (not including newline)
    const fn is_whitespace(ch: char) -> bool {
        matches!(ch, ' ' | '\t' | '\x0B' | '\x0C')
    }

    /// Lex consecutive whitespace characters (not including newlines) into a single `Whitespace` token
    fn lex_whitespace(&mut self) -> Result<Spanned<'src, PreprocessingToken>, PpLexerError<'src>> {
        let first = self.read_char()?.unwrap();
        let mut span = first.span;

        while let Some(ch) = self.peek_char()? {
            if Self::is_whitespace(ch.value) {
                let ch = self.read_char()?.unwrap();
                span.end = ch.span.end;
            } else {
                break;
            }
        }

        Ok(Spanned::new(PreprocessingToken::Whitespace, span))
    }

    /// Lex a newline character into a `Newline` token
    fn lex_newline(&mut self) -> Result<Spanned<'src, PreprocessingToken>, PpLexerError<'src>> {
        let ch = self.read_char()?.unwrap();
        assert_eq!(ch.value, '\n');
        Ok(Spanned::new(PreprocessingToken::Newline, ch.span))
    }

    /// Lex a line comment ("//" until newline) as a `Whitespace` token
    fn lex_line_comment(
        &mut self,
    ) -> Result<Spanned<'src, PreprocessingToken>, PpLexerError<'src>> {
        let start_span = self.read_known_char('/')?.unwrap();
        let mut end_span = self.read_known_char('/')?.unwrap();

        while let Some(ch) = self.peek_char()? {
            if ch.value == '\n' {
                break;
            }
            end_span = self.read_char()?.unwrap().span;
        }

        let span = Span::new(start_span.file, start_span.start, end_span.end);
        Ok(Spanned::new(PreprocessingToken::Whitespace, span))
    }

    /// Lex a block comment ("/*" to "*/") as a `Whitespace` token
    fn lex_block_comment(
        &mut self,
    ) -> Result<Spanned<'src, PreprocessingToken>, PpLexerError<'src>> {
        let start_span = self.read_known_char('/')?.unwrap();
        self.read_known_char('*')?;

        // Read until "*/" or EOF
        loop {
            let Some(ch) = self.read_char()? else {
                return Err(PpLexerError::UnterminatedBlockComment(start_span));
            };

            if ch.value == '*'
                && let Some(next) = self.peek_char()?
                && next.value == '/'
            {
                let end_span = self.read_known_char('/')?.unwrap();
                let span = Span::new(start_span.file, start_span.start, end_span.end);
                return Ok(Spanned::new(PreprocessingToken::Whitespace, span));
            }
        }
    }

    /// Read a pp-number
    fn read_pp_number(&mut self) -> Result<Spanned<'src, String>, PpLexerError<'src>> {
        let mut chars = String::new();
        let first = self.read_char()?.unwrap();
        let mut span = first.span;
        chars.push(first.value);

        while let Some(peek) = self.peek_char()? {
            let pv = peek.value;

            if matches!(pv, 'e' | 'E' | 'p' | 'P') {
                // Consume the exponent indicator, then optionally a sign
                let exp = self.read_char()?.unwrap();
                span.end = exp.span.end;
                chars.push(exp.value);
                if let Some(next) = self.peek_char()?
                    && matches!(next.value, '+' | '-')
                {
                    let sign = self.read_char()?.unwrap();
                    span.end = sign.span.end;
                    chars.push(sign.value);
                }
            } else if pv == '.' || pv.is_ascii_digit() || Self::is_identifier_continue(pv) {
                let ch = self.read_char()?.unwrap();
                span.end = ch.span.end;
                chars.push(ch.value);
            } else {
                break;
            }
        }

        Ok(Spanned::new(chars, span))
    }

    /// Read an identifier
    fn read_identifier(&mut self) -> Result<Spanned<'src, Identifier>, PpLexerError<'src>> {
        let mut chars = String::new();
        let first = self.read_char()?.unwrap();
        let mut span = first.span;
        chars.push(first.value);

        while let Some(ch) = self.peek_char()? {
            if Self::is_identifier_continue(ch.value) {
                let ch = self.read_char()?.unwrap();
                span.end = ch.span.end;
                chars.push(ch.value);
            } else {
                break;
            }
        }

        let identifier = Identifier::new(&chars).unwrap();

        Ok(Spanned::new(identifier, span))
    }

    /// Helper to consume N additional characters and create a span from first to last
    fn consume_and_span(
        &mut self,
        first: Spanned<'src, char>,
        count: usize,
    ) -> Result<Span<'src>, PpLexerError<'src>> {
        let mut last = first;
        for _ in 0..count {
            last = self.read_char()?.unwrap();
        }
        Ok(Span::new(first.span.file, first.span.start, last.span.end))
    }

    /// Helper to create a multi-char punctuator
    fn mk_punct(
        &mut self,
        first: Spanned<'src, char>,
        count: usize,
        punct: Punctuator,
    ) -> Result<Spanned<'src, Punctuator>, PpLexerError<'src>> {
        let span = self.consume_and_span(first, count)?;
        Ok(Spanned::new(punct, span))
    }

    /// Read a punctuator with longest match
    fn read_punctuator(&mut self) -> Result<Spanned<'src, Punctuator>, PpLexerError<'src>> {
        let first = self.read_char()?.unwrap();

        // Copy values of next 3 chars
        let vals = [
            self.peek_n(0)?.map(|ch| ch.value),
            self.peek_n(1)?.map(|ch| ch.value),
            self.peek_n(2)?.map(|ch| ch.value),
        ];

        match (first.value, vals[0], vals[1], vals[2]) {
            // 4-char digraph:
            ('%', Some(':'), Some('%'), Some(':')) => self.mk_punct(first, 3, Punctuator::HashHash),
            // 3-char operators:
            ('<', Some('<'), Some('='), _) => self.mk_punct(first, 2, Punctuator::LShiftAssign),
            ('>', Some('>'), Some('='), _) => self.mk_punct(first, 2, Punctuator::RShiftAssign),
            ('.', Some('.'), Some('.'), _) => self.mk_punct(first, 2, Punctuator::Ellips),
            // 2-char operators and digraphs:
            ('<', Some(':'), _, _) => self.mk_punct(first, 1, Punctuator::LBracket),
            (':', Some('>'), _, _) => self.mk_punct(first, 1, Punctuator::RBracket),
            ('<', Some('%'), _, _) => self.mk_punct(first, 1, Punctuator::LCurly),
            ('%', Some('>'), _, _) => self.mk_punct(first, 1, Punctuator::RCurly),
            ('-', Some('>'), _, _) => self.mk_punct(first, 1, Punctuator::Arrow),
            ('+', Some('+'), _, _) => self.mk_punct(first, 1, Punctuator::Incr),
            ('-', Some('-'), _, _) => self.mk_punct(first, 1, Punctuator::Decr),
            ('<', Some('<'), _, _) => self.mk_punct(first, 1, Punctuator::LShift),
            ('>', Some('>'), _, _) => self.mk_punct(first, 1, Punctuator::RShift),
            ('<', Some('='), _, _) => self.mk_punct(first, 1, Punctuator::Leq),
            ('>', Some('='), _, _) => self.mk_punct(first, 1, Punctuator::Geq),
            ('=', Some('='), _, _) => self.mk_punct(first, 1, Punctuator::Eq),
            ('!', Some('='), _, _) => self.mk_punct(first, 1, Punctuator::Neq),
            ('&', Some('&'), _, _) => self.mk_punct(first, 1, Punctuator::And),
            ('|', Some('|'), _, _) => self.mk_punct(first, 1, Punctuator::Or),
            ('*', Some('='), _, _) => self.mk_punct(first, 1, Punctuator::StarAssign),
            ('/', Some('='), _, _) => self.mk_punct(first, 1, Punctuator::SlashAssign),
            ('%', Some('='), _, _) => self.mk_punct(first, 1, Punctuator::PercentAssign),
            ('+', Some('='), _, _) => self.mk_punct(first, 1, Punctuator::PlusAssign),
            ('-', Some('='), _, _) => self.mk_punct(first, 1, Punctuator::MinusAssign),
            ('^', Some('='), _, _) => self.mk_punct(first, 1, Punctuator::BitXorAssign),
            ('|', Some('='), _, _) => self.mk_punct(first, 1, Punctuator::BitOrAssign),
            ('&', Some('='), _, _) => self.mk_punct(first, 1, Punctuator::BitAndAssign),
            ('%', Some(':'), _, _) => self.mk_punct(first, 1, Punctuator::Hash),
            ('#', Some('#'), _, _) => self.mk_punct(first, 1, Punctuator::HashHash),
            ('[', _, _, _) => Ok(Spanned::new(Punctuator::LBracket, first.span)),
            (']', _, _, _) => Ok(Spanned::new(Punctuator::RBracket, first.span)),
            ('(', _, _, _) => Ok(Spanned::new(Punctuator::LParen, first.span)),
            (')', _, _, _) => Ok(Spanned::new(Punctuator::RParen, first.span)),
            ('{', _, _, _) => Ok(Spanned::new(Punctuator::LCurly, first.span)),
            ('}', _, _, _) => Ok(Spanned::new(Punctuator::RCurly, first.span)),
            ('.', _, _, _) => Ok(Spanned::new(Punctuator::Dot, first.span)),
            ('&', _, _, _) => Ok(Spanned::new(Punctuator::Amp, first.span)),
            ('*', _, _, _) => Ok(Spanned::new(Punctuator::Asterisk, first.span)),
            ('+', _, _, _) => Ok(Spanned::new(Punctuator::Plus, first.span)),
            ('-', _, _, _) => Ok(Spanned::new(Punctuator::Minus, first.span)),
            ('~', _, _, _) => Ok(Spanned::new(Punctuator::Tilde, first.span)),
            ('!', _, _, _) => Ok(Spanned::new(Punctuator::Exclamation, first.span)),
            ('/', _, _, _) => Ok(Spanned::new(Punctuator::Slash, first.span)),
            ('%', _, _, _) => Ok(Spanned::new(Punctuator::Percent, first.span)),
            ('<', _, _, _) => Ok(Spanned::new(Punctuator::Lt, first.span)),
            ('>', _, _, _) => Ok(Spanned::new(Punctuator::Gt, first.span)),
            ('^', _, _, _) => Ok(Spanned::new(Punctuator::BitXor, first.span)),
            ('|', _, _, _) => Ok(Spanned::new(Punctuator::BitOr, first.span)),
            ('?', _, _, _) => Ok(Spanned::new(Punctuator::Question, first.span)),
            (':', _, _, _) => Ok(Spanned::new(Punctuator::Colon, first.span)),
            (';', _, _, _) => Ok(Spanned::new(Punctuator::Semicolon, first.span)),
            ('=', _, _, _) => Ok(Spanned::new(Punctuator::Assign, first.span)),
            (',', _, _, _) => Ok(Spanned::new(Punctuator::Comma, first.span)),
            ('#', _, _, _) => Ok(Spanned::new(Punctuator::Hash, first.span)),
            _ => Err(PpLexerError::UnexpectedChar(first)),
        }
    }
}

impl<'src> Iterator for PpLexer<'src> {
    type Item = Result<Spanned<'src, PreprocessingToken>, PpLexerError<'src>>;

    fn next(&mut self) -> Option<Self::Item> {
        // Check pending buffer first
        if let Some(token) = self.pending.take() {
            return Some(Ok(token));
        }

        // Peek next char
        let ch = match self.peek_char() {
            Ok(Some(c)) => *c,
            Ok(None) => return None, // EOF
            Err(e) => return Some(Err(e)),
        };

        // Check for newline first
        if ch.value == '\n' {
            return Some(self.lex_newline());
        }

        // Check for whitespace (not including newlines)
        if Self::is_whitespace(ch.value) {
            return Some(self.lex_whitespace());
        }

        // Check for comments (treated as whitespace)
        if ch.value == '/'
            && let Ok(Some(next)) = self.peek_n(1)
        {
            if next.value == '/' {
                return Some(self.lex_line_comment());
            }
            if next.value == '*' {
                return Some(self.lex_block_comment());
            }
        }

        // Try identifier
        if Self::is_identifier_start(ch.value) {
            return Some(
                self.read_identifier()
                    .map(|id| Spanned::new(PreprocessingToken::Identifier(id.value), id.span)),
            );
        }

        // Try pp-number (digit or . digit)
        if Self::is_pp_number_start(ch.value)
            || (ch.value == '.'
                && matches!(self.peek_n(1), Ok(Some(next)) if next.value.is_ascii_digit()))
        {
            return Some(
                self.read_pp_number()
                    .map(|num| Spanned::new(PreprocessingToken::PpNumber(num.value), num.span)),
            );
        }

        // Try punctuator
        match self.read_punctuator() {
            Ok(p) => Some(Ok(Spanned::new(
                PreprocessingToken::Punctuator(p.value),
                p.span,
            ))),
            Err(PpLexerError::UnexpectedChar(ch)) => {
                // Fall back to `OtherChar` for any character not matching other categories
                Some(Ok(Spanned::new(
                    PreprocessingToken::OtherChar(ch.value),
                    ch.span,
                )))
            }
            Err(e) => Some(Err(e)),
        }
    }
}
