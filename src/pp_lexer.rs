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

    /// Skip whitespace characters (including newlines)
    fn skip_whitespace(&mut self) -> Result<(), PpLexerError<'src>> {
        while let Some(ch) = self.peek_char()? {
            match ch.value {
                ' ' | '\t' | '\n' | '\x0B' | '\x0C' => {
                    self.read_char()?;
                }
                _ => break,
            }
        }
        Ok(())
    }

    /// Skip a line comment ("//" until newline)
    fn skip_line_comment(&mut self) -> Result<(), PpLexerError<'src>> {
        self.read_known_char('/')?;
        self.read_known_char('/')?;

        while let Some(ch) = self.peek_char()? {
            if ch.value == '\n' {
                break;
            }
            self.read_char()?;
        }

        Ok(())
    }

    /// Skip a block comment ("/*" to "*/")
    fn skip_block_comment(&mut self) -> Result<(), PpLexerError<'src>> {
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
                self.read_known_char('/')?;
                break;
            }
        }

        Ok(())
    }

    /// Read a pp-number
    fn read_pp_number(&mut self) -> Result<Spanned<'src, String>, PpLexerError<'src>> {
        // TODO: finish implementation?

        // [digit]
        // . [digit]
        // [pp-number] [digit]
        // [pp-number] [identifier-nondigit]
        // [pp-number] e [sign]
        // [pp-number] E [sign]
        // [pp-number] p [sign]
        // [pp-number] P [sign]
        // [pp-number] .
        let mut chars = String::new();
        let first = self.read_char()?.unwrap();
        let start = first.span.start;
        chars.push(first.value);

        while let Some(ch) = self.peek_char()? {
            if Self::is_identifier_continue(ch.value) {
                chars.push(self.read_char()?.unwrap().value);
            } else {
                break;
            }
        }

        let end = self.lookahead.front().map_or_else(
            #[allow(clippy::cast_possible_truncation)]
            || crate::span::ByteOffset(start.0 + chars.len() as u32),
            |ch| ch.span.start,
        );

        let span = Span::new(first.span.file, start, end);

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

        loop {
            // Peek next char
            let ch = match self.peek_char() {
                Ok(Some(c)) => *c,
                Ok(None) => return None, // EOF
                Err(e) => return Some(Err(e)),
            };

            // Skip whitespace (including newlines)
            if matches!(ch.value, ' ' | '\t' | '\n' | '\x0B' | '\x0C') {
                if let Err(e) = self.skip_whitespace() {
                    return Some(Err(e));
                }
                continue;
            }

            // Check for comments
            if ch.value == '/'
                && let Ok(Some(next)) = self.peek_n(1)
            {
                if next.value == '/' {
                    if let Err(e) = self.skip_line_comment() {
                        return Some(Err(e));
                    }
                    continue;
                }
                if next.value == '*' {
                    if let Err(e) = self.skip_block_comment() {
                        return Some(Err(e));
                    }
                    continue;
                }
            }

            // Try identifier
            if Self::is_identifier_start(ch.value) {
                return Some(
                    self.read_identifier()
                        .map(|id| Spanned::new(PreprocessingToken::Identifier(id.value), id.span)),
                );
            }

            // Try pp-number
            if Self::is_pp_number_start(ch.value) {
                return Some(
                    self.read_pp_number()
                        .map(|num| Spanned::new(PreprocessingToken::PpNumber(num.value), num.span)),
                );
            }

            // Try punctuator
            return Some(
                self.read_punctuator()
                    .map(|p| Spanned::new(PreprocessingToken::Punctuator(p.value), p.span)),
            );
        }
    }
}
