use crate::lex::token::{Keyword, Punct, TokenKind};
use crate::lex::{LexError, LexErrorKind, token::Token};
use crate::{FileId, Span, Spanned};

#[derive(Debug, Clone)]
pub(super) struct Lexer<'a> {
    file: FileId,
    input: &'a str,
    pos: usize,
    pub(super) lines: Vec<usize>,
    pub(super) tokens: Vec<Token>,
    pub(super) errors: Vec<LexError>,
}

impl<'a> Lexer<'a> {
    pub(super) fn new(file: FileId, input: &'a str) -> Self {
        Self {
            file,
            input,
            pos: 0,
            lines: vec![0],
            tokens: Vec::new(),
            errors: Vec::new(),
        }
    }

    #[inline]
    fn next(&mut self) -> u8 {
        self.pos += 1;
        self.input.as_bytes()[self.pos - 1]
    }

    #[inline]
    fn next_eq(&mut self, byte: u8) -> bool {
        let cond = self.pos < self.input.len() && self.input.as_bytes()[self.pos] == byte;
        if cond {
            self.pos += 1;
        }
        cond
    }

    fn skip_ws(&mut self) {
        loop {
            if self.next_eq(b'\n') {
                self.lines.push(self.pos);
                continue;
            }
            if self.next_eq(b' ') || self.next_eq(b'\t') || self.next_eq(b'\r') {
                continue;
            }
            break;
        }
    }

    fn push_token(&mut self, token: TokenKind, start: usize) {
        self.tokens.push(Spanned {
            value: token,
            span: Span {
                file: self.file,
                start,
                end: self.pos,
            },
        });
    }

    fn push_error(&mut self, error: LexErrorKind, start: usize) {
        self.errors.push(Spanned {
            value: error,
            span: Span {
                file: self.file,
                start,
                end: self.pos,
            },
        });
    }

    pub(super) fn lex(&mut self) {
        while self.pos < self.input.len() {
            self.skip_ws();
            if self.pos == self.input.len() {
                break;
            }

            let first = self.next();
            match first {
                b'!' | b'%' | b'&' | b'(' | b')' | b'*' | b'+' | b',' | b'-' | b'.' | b'/'
                | b';' | b'<' | b'=' | b'>' | b'[' | b']' | b'{' | b'|' | b'}' => {
                    self.lex_punct_comment(first);
                }
                b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.lex_kw_ident(),
                b'0'..=b'9' => self.lex_num(),
                b'"' => self.lex_string(),
                _ => self.lex_unexpected(),
            }
        }
        if !self.input.is_empty() && !self.input.ends_with('\n') {
            self.push_error(LexErrorKind::MissingFinalNewline, self.pos);
        }
        self.lines.push(self.pos);
    }

    fn lex_punct_comment(&mut self, first: u8) {
        #[allow(clippy::enum_glob_use)]
        use self::Punct::*;
        use TokenKind::Punct;

        let start = self.pos - 1;
        match first {
            b'/' if self.next_eq(b'/') => {
                while self.pos < self.input.len() && self.input.as_bytes()[self.pos] != b'\n' {
                    self.pos += 1;
                }
            }
            b'/' if self.next_eq(b'*') => {
                while self.pos < self.input.len() {
                    if self.next_eq(b'*') && self.next_eq(b'/') {
                        break;
                    }
                    if self.input.as_bytes()[self.pos] == b'\n' {
                        self.pos += 1;
                        self.lines.push(self.pos);
                    } else {
                        self.pos += 1;
                    }
                }
            }
            //
            b'(' => self.push_token(Punct(Lparen), start),
            b')' => self.push_token(Punct(Rparen), start),
            b'[' => self.push_token(Punct(Lsquare), start),
            b']' => self.push_token(Punct(Rsquare), start),
            b'{' => self.push_token(Punct(Lcurly), start),
            b'}' => self.push_token(Punct(Rcurly), start),
            b',' => self.push_token(Punct(Comma), start),
            b';' => self.push_token(Punct(Semicolon), start),
            b'.' => self.push_token(Punct(Dot), start),
            //
            b'*' if self.next_eq(b'=') => self.push_token(Punct(StarEq), start),
            b'*' => self.push_token(Punct(Star), start),
            b'/' if self.next_eq(b'=') => self.push_token(Punct(SlashEq), start),
            b'/' => self.push_token(Punct(Slash), start),
            b'%' if self.next_eq(b'=') => self.push_token(Punct(PercentEq), start),
            b'%' => self.push_token(Punct(Percent), start),
            b'+' if self.next_eq(b'=') => self.push_token(Punct(PlusEq), start),
            b'+' => self.push_token(Punct(Plus), start),
            b'-' if self.next_eq(b'=') => self.push_token(Punct(MinusEq), start),
            b'-' => self.push_token(Punct(Minus), start),
            //
            b'<' if self.next_eq(b'<') => {
                if self.next_eq(b'=') {
                    self.push_token(Punct(LshiftEq), start);
                } else {
                    self.push_token(Punct(Lshift), start);
                }
            }
            b'>' if self.next_eq(b'>') => {
                if self.next_eq(b'=') {
                    self.push_token(Punct(RshiftEq), start);
                } else {
                    self.push_token(Punct(Rshift), start);
                }
            }
            //
            b'=' if self.next_eq(b'=') => self.push_token(Punct(EqEq), start),
            b'=' => self.push_token(Punct(Eq), start),
            b'!' if self.next_eq(b'=') => self.push_token(Punct(BangEq), start),
            b'!' => self.push_token(Punct(Bang), start),
            b'>' if self.next_eq(b'=') => self.push_token(Punct(GtEq), start),
            b'>' => self.push_token(Punct(Gt), start),
            b'<' if self.next_eq(b'=') => self.push_token(Punct(LtEq), start),
            b'<' => self.push_token(Punct(Lt), start),
            //
            b'&' if self.next_eq(b'&') => self.push_token(Punct(And), start),
            b'|' if self.next_eq(b'|') => self.push_token(Punct(Or), start),
            //
            _ => unreachable!(),
        }
    }

    fn lex_kw_ident(&mut self) {
        #[allow(clippy::enum_glob_use)]
        use self::Keyword::*;
        use TokenKind::{Ident, Keyword};

        let start = self.pos - 1;
        while self.pos < self.input.len()
            && matches!(self.input.as_bytes()[self.pos], b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_')
        {
            self.pos += 1;
        }
        let contents = self.input[start..self.pos].to_owned();
        match &contents as &str {
            "struct" => self.push_token(Keyword(Struct), start),
            "if" => self.push_token(Keyword(If), start),
            "else" => self.push_token(Keyword(Else), start),
            "for" => self.push_token(Keyword(For), start),
            "do" => self.push_token(Keyword(Do), start),
            "while" => self.push_token(Keyword(While), start),
            "return" => self.push_token(Keyword(Return), start),
            "continue" => self.push_token(Keyword(Continue), start),
            "break" => self.push_token(Keyword(Break), start),
            "switch" => self.push_token(Keyword(Switch), start),
            "case" => self.push_token(Keyword(Case), start),
            _ => self.push_token(Ident(contents), start),
        }
    }

    fn lex_num(&mut self) {
        let start = self.pos - 1;
        while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_digit() {
            self.pos += 1;
        }

        let is_decimal = self.pos + 1 < self.input.len()
            && self.input.as_bytes()[self.pos] == b'.'
            && self.input.as_bytes()[self.pos + 1].is_ascii_digit();
        if is_decimal {
            self.pos += 1;
            while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
            let contents = &self.input[start..self.pos];
            self.push_token(
                TokenKind::Decimal(contents.parse().expect("validated decimal")),
                start,
            );
        } else {
            let contents = &self.input[start..self.pos];
            self.push_token(
                TokenKind::Integer(contents.parse().expect("validated integer")),
                start,
            );
        }
    }

    fn lex_string(&mut self) {
        let start = self.pos - 1;
        let mut contents = String::new();
        let mut segment_start = self.pos;

        while self.pos < self.input.len() {
            match self.next() {
                b'"' => {
                    contents.push_str(&self.input[segment_start..self.pos - 1]);
                    self.push_token(TokenKind::String(contents), start);
                    return;
                }
                b'\\' => {
                    contents.push_str(&self.input[segment_start..self.pos - 1]);
                    if self.pos >= self.input.len() {
                        break;
                    }
                    let escape_start = self.pos - 1;
                    match self.next() {
                        b'\\' => contents.push('\\'),
                        b'"' => contents.push('"'),
                        b'n' => contents.push('\n'),
                        b'r' => contents.push('\r'),
                        b't' => contents.push('\t'),
                        _ => {
                            let escaped = self.input[escape_start + 1..]
                                .chars()
                                .next()
                                .expect("not at eof");
                            self.pos = escape_start + 1 + escaped.len_utf8();
                            self.push_error(
                                LexErrorKind::InvalidStringEscape(escaped),
                                escape_start,
                            );
                        }
                    }
                    segment_start = self.pos;
                }
                b'\n' => self.lines.push(self.pos),
                _ => {}
            }
        }

        self.push_error(LexErrorKind::UnterminatedString, start);
    }

    fn lex_unexpected(&mut self) {
        let start = self.pos - 1;
        let ch = self.input[start..].chars().next().expect("not at eof");
        self.pos = start + ch.len_utf8();
        self.push_error(LexErrorKind::UnexpectedChar(ch), start);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex_input(input: &str) -> Lexer<'_> {
        let mut lexer = Lexer::new(7, input);
        lexer.lex();
        lexer
    }

    fn assert_values(input: &str, expected: &[TokenKind]) {
        let lexer = lex_input(input);
        assert_eq!(
            lexer
                .tokens
                .iter()
                .map(|token| &token.value)
                .collect::<Vec<_>>(),
            expected.iter().collect::<Vec<_>>(),
            "input: {input:?}"
        );
    }

    fn assert_spans(input: &str, expected: &[(usize, usize)]) {
        let lexer = lex_input(input);
        assert_eq!(
            lexer
                .tokens
                .iter()
                .map(|token| (token.span.start, token.span.end))
                .collect::<Vec<_>>(),
            expected,
            "input: {input:?}"
        );
        assert!(lexer.tokens.iter().all(|token| token.span.file == 7));
    }

    #[test]
    fn empty_input_has_no_tokens() {
        assert_values("", &[]);
    }

    #[test]
    fn whitespace_only_input_has_no_tokens() {
        assert_values(" \t\r\n  \n", &[]);
    }

    #[test]
    fn all_single_character_punctuation_is_lexed() {
        use self::Punct as P;
        use P::*;
        use TokenKind::Punct;

        assert_values(
            "()[]{},;.=!><*/%+-",
            &[
                Punct(Lparen),
                Punct(Rparen),
                Punct(Lsquare),
                Punct(Rsquare),
                Punct(Lcurly),
                Punct(Rcurly),
                Punct(Comma),
                Punct(Semicolon),
                Punct(Dot),
                Punct(Eq),
                Punct(Bang),
                Punct(Gt),
                Punct(Lt),
                Punct(Star),
                Punct(Slash),
                Punct(Percent),
                Punct(Plus),
                Punct(Minus),
            ],
        );
    }

    #[test]
    fn arithmetic_assignment_punctuation_is_lexed() {
        use self::Punct as P;
        use P::*;
        use TokenKind::Punct;

        assert_values(
            "*= /= %= += -=",
            &[
                Punct(StarEq),
                Punct(SlashEq),
                Punct(PercentEq),
                Punct(PlusEq),
                Punct(MinusEq),
            ],
        );
    }

    #[test]
    fn comparison_and_shift_punctuation_is_lexed() {
        use self::Punct as P;
        use P::*;
        use TokenKind::Punct;

        assert_values(
            "<< <<= >> >>= == != >= <= && ||",
            &[
                Punct(Lshift),
                Punct(LshiftEq),
                Punct(Rshift),
                Punct(RshiftEq),
                Punct(EqEq),
                Punct(BangEq),
                Punct(GtEq),
                Punct(LtEq),
                Punct(And),
                Punct(Or),
            ],
        );
    }

    #[test]
    fn punctuation_spans_include_the_second_character() {
        use self::Punct as P;
        use P::*;
        use TokenKind::Punct;

        assert_values(
            "*= << >>= !=",
            &[Punct(StarEq), Punct(Lshift), Punct(RshiftEq), Punct(BangEq)],
        );
        assert_spans("*= << >>= !=", &[(0, 2), (3, 5), (6, 9), (10, 12)]);
    }

    #[test]
    fn every_keyword_is_distinguished_from_an_identifier() {
        use self::Keyword::*;
        use TokenKind::Keyword;

        assert_values(
            "struct if else for do while return continue break switch case",
            &[
                Keyword(Struct),
                Keyword(If),
                Keyword(Else),
                Keyword(For),
                Keyword(Do),
                Keyword(While),
                Keyword(Return),
                Keyword(Continue),
                Keyword(Break),
                Keyword(Switch),
                Keyword(Case),
            ],
        );
    }

    #[test]
    fn identifiers_allow_digits_and_underscores_after_the_first_character() {
        use TokenKind::Ident;

        assert_values(
            "x x2 _private snake_case CamelCase value_123",
            &[
                Ident("x".into()),
                Ident("x2".into()),
                Ident("_private".into()),
                Ident("snake_case".into()),
                Ident("CamelCase".into()),
                Ident("value_123".into()),
            ],
        );
    }

    #[test]
    fn identifiers_and_punctuation_have_source_spans() {
        use self::Keyword::If;
        use self::Punct as P;
        use P::*;
        use TokenKind::Punct;
        use TokenKind::{Ident, Keyword};

        assert_values("foo + if", &[Ident("foo".into()), Punct(Plus), Keyword(If)]);
        assert_spans("foo + if", &[(0, 3), (4, 5), (6, 8)]);
    }

    #[test]
    fn a_line_comment_at_end_of_input_is_discarded() {
        assert_values("// comment until eof", &[]);
    }

    #[test]
    fn consecutive_line_comments_are_discarded() {
        assert_values("// first// second", &[]);
        assert_values("// first\n// second", &[]);
    }

    #[test]
    fn whitespace_between_tokens_is_ignored() {
        use self::Punct as P;
        use P::*;
        use TokenKind::Punct;

        assert_values("(\n\t+\r)", &[Punct(Lparen), Punct(Plus), Punct(Rparen)]);
    }

    #[test]
    fn line_comment_can_be_followed_by_more_source() {
        use self::Punct as P;
        use P::*;
        use TokenKind::Punct;

        assert_values("// comment\n+", &[Punct(Plus)]);
    }

    #[test]
    fn block_comments_are_discarded() {
        use self::Punct as P;
        use P::*;
        use TokenKind::Punct;

        assert_values("/* comment */ +", &[Punct(Plus)]);
    }

    #[test]
    fn integers_are_lexed() {
        use TokenKind::Integer;

        assert_values(
            "0 1 42 18446744073709551615",
            &[Integer(0), Integer(1), Integer(42), Integer(u64::MAX)],
        );
    }

    #[test]
    fn decimal_numbers_are_lexed() {
        use TokenKind::Decimal;

        assert_values(
            "0.0 1.5 42.125",
            &[Decimal(0.0), Decimal(1.5), Decimal(42.125)],
        );
    }

    #[test]
    fn strings_are_unescaped_and_unquoted() {
        use TokenKind::String;

        assert_values(
            r#""hello" "with spaces" "escaped \"quote\"""#,
            &[
                String("hello".into()),
                String("with spaces".into()),
                String("escaped \"quote\"".into()),
            ],
        );
    }

    #[test]
    fn empty_strings_are_valid() {
        use TokenKind::String;

        assert_values("\"\"", &[String(std::string::String::new())]);
    }

    #[test]
    fn string_spans_are_utf8_byte_offsets() {
        use TokenKind::String;

        assert_values("\"h\u{e9}\"", &[String("h\u{e9}".into())]);
        assert_spans("\"h\u{e9}\"", &[(0, 5)]);
    }

    #[test]
    fn a_missing_string_terminator_records_an_error() {
        let mut lexer = Lexer::new(7, "\"unterminated");
        lexer.lex();

        assert!(lexer.tokens.is_empty());
        assert_eq!(lexer.errors.len(), 2);
        assert_eq!(lexer.errors[0].value, LexErrorKind::UnterminatedString);
        assert_eq!(lexer.errors[1].value, LexErrorKind::MissingFinalNewline);
    }

    #[test]
    fn invalid_string_escapes_record_an_error() {
        let mut lexer = Lexer::new(7, "\"bad\\q\"\n");
        lexer.lex();

        assert_eq!(lexer.errors.len(), 1);
        assert_eq!(
            lexer.errors[0].value,
            LexErrorKind::InvalidStringEscape('q')
        );
        assert_eq!(lexer.errors[0].span.start, 4);
        assert_eq!(lexer.errors[0].span.end, 6);
    }

    #[test]
    fn a_string_ending_with_an_escape_is_unterminated() {
        let mut lexer = Lexer::new(7, "\"unterminated\\");
        lexer.lex();

        assert_eq!(lexer.errors[0].value, LexErrorKind::UnterminatedString);
    }

    #[test]
    fn unexpected_characters_become_lexer_errors() {
        let mut lexer = Lexer::new(7, "@");
        lexer.lex();

        assert_eq!(lexer.errors.len(), 2);
        assert_eq!(lexer.errors[0].value, LexErrorKind::UnexpectedChar('@'));
        assert_eq!(lexer.errors[1].value, LexErrorKind::MissingFinalNewline);
        assert_eq!(lexer.errors[1].span.start, 1);
        assert_eq!(lexer.errors[1].span.end, 1);
    }

    #[test]
    fn a_final_newline_is_required() {
        let mut lexer = Lexer::new(7, "identifier");
        lexer.lex();

        assert_eq!(lexer.errors.len(), 1);
        assert_eq!(lexer.errors[0].value, LexErrorKind::MissingFinalNewline);
        assert_eq!(lexer.errors[0].span.start, 10);
        assert_eq!(lexer.errors[0].span.end, 10);
    }
}
