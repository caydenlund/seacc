use crate::span::{ByteOffset, Span, Spanned};
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::str;

mod error;
pub use error::SourceError;

/// A buffered source file reader that performs trigraph sequence replacement
///
/// The reader yields one character at a time with an associated `Span`.
pub struct SourceReader<'src> {
    /// The file path reference
    file_path: &'src str,
    /// Buffered reader for the underlying file
    reader: BufReader<File>,
    /// Current buffer of bytes from the file
    buffer: Vec<u8>,
    /// Current position in the buffer
    pos: usize,
    /// Current byte offset in the original file
    byte_offset: u32,
    /// Whether we've reached the end of the file
    eof: bool,
    /// Raw character lookahead buffer (for trigraph/CRLF detection, max 3 chars)
    lookahead: VecDeque<Spanned<'src, char>>,
    /// Pending character from pass 2 lookahead
    pending: Option<Spanned<'src, char>>,
    /// Last character output from pass 2 (for EOF validation)
    last_output_char: Option<char>,
}

impl<'src> SourceReader<'src> {
    /// Creates a new `SourceReader` from a file path
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or read.
    pub fn new(file_path: &'src str) -> Result<Self, SourceError<'src>> {
        let file = File::open(Path::new(file_path))?;
        let reader = BufReader::new(file);

        Ok(Self {
            file_path,
            reader,
            buffer: Vec::new(),
            pos: 0,
            byte_offset: 0,
            eof: false,
            lookahead: VecDeque::new(),
            pending: None,
            last_output_char: None,
        })
    }

    /// Reads the next UTF-8 character from the buffer, refilling if necessary
    /// Checks `lookahead` first before reading from the file
    fn read_raw_char(&mut self) -> Result<Option<Spanned<'src, char>>, SourceError<'src>> {
        // Check raw lookahead first
        if let Some(ch) = self.lookahead.pop_front() {
            return Ok(Some(ch));
        }

        loop {
            if self.pos >= self.buffer.len() {
                if self.eof {
                    return Ok(None);
                }

                self.buffer.clear();
                let bytes_read = self.reader.read_until(b'\n', &mut self.buffer)?;

                if bytes_read == 0 {
                    self.eof = true;
                    return Ok(None);
                }

                self.pos = 0;
            }

            let start_offset = ByteOffset(self.byte_offset);

            // Try to decode a UTF-8 character
            let remaining = &self.buffer[self.pos..];

            let char_len = if remaining[0] & 0b1000_0000 == 0 {
                // ASCII (1 byte)
                1
            } else if remaining[0] & 0b1110_0000 == 0b1100_0000 {
                // 2-byte sequence
                2
            } else if remaining[0] & 0b1111_0000 == 0b1110_0000 {
                // 3-byte sequence
                3
            } else if remaining[0] & 0b1111_1000 == 0b1111_0000 {
                // 4-byte sequence
                4
            } else {
                // Invalid UTF-8
                self.pos += 1;
                self.byte_offset += 1;
                let end_offset = ByteOffset(self.byte_offset);
                let span = Span::new(self.file_path, start_offset, end_offset);
                return Err(SourceError::InvalidUtf8(span));
            };

            if remaining.len() < char_len {
                // Need to read more
                if self.eof {
                    // Invalid UTF-8
                    self.pos += 1;
                    self.byte_offset += 1;
                    let end_offset = ByteOffset(self.byte_offset);
                    let span = Span::new(self.file_path, start_offset, end_offset);
                    return Err(SourceError::InvalidUtf8(span));
                }
                // Refill buffer and try again
                continue;
            }

            // Try to decode the UTF-8 sequence
            if let Ok(s) = str::from_utf8(&remaining[..char_len]) {
                let ch = s.chars().next().unwrap();
                self.pos += char_len;
                self.byte_offset += u32::try_from(char_len).unwrap();
                let end_offset = ByteOffset(self.byte_offset);
                let span = Span::new(self.file_path, start_offset, end_offset);
                return Ok(Some(Spanned::new(ch, span)));
            }

            // Invalid UTF-8
            self.pos += 1;
            self.byte_offset += 1;
            let end_offset = ByteOffset(self.byte_offset);
            let span = Span::new(self.file_path, start_offset, end_offset);
            return Err(SourceError::InvalidUtf8(span));
        }
    }

    /// Attempts to match and replace a trigraph sequence starting with "??"
    ///
    /// Returns the replacement character if a trigraph is found.
    /// If not a trigraph, returns the characters that should be output instead (in order).
    fn match_trigraph(
        &mut self,
        first: Spanned<'src, char>,
    ) -> Result<Option<Spanned<'src, char>>, SourceError<'src>> {
        debug_assert_eq!(first.value, '?');

        // Try to read the second '?'
        let Some(second) = self.read_raw_char()? else {
            self.lookahead.push_back(first);
            return Ok(None);
        };

        if second.value != '?' {
            self.lookahead.extend(&[first, second]);
            return Ok(None);
        }

        // Try to read the third character
        let Some(third) = self.read_raw_char()? else {
            self.lookahead.extend(&[first, second]);
            return Ok(None);
        };

        // Check if it's a valid trigraph and return replacement
        let replacement_char = match third.value {
            '=' => '#',
            '(' => '[',
            '/' => '\\',
            ')' => ']',
            '\'' => '^',
            '<' => '{',
            '!' => '|',
            '>' => '}',
            '-' => '~',
            _ => {
                self.lookahead.extend(&[first, second, third]);
                return Ok(None);
            }
        };

        // Valid trigraph found - create a spanned char with the span covering all 3 chars
        let span = Span::new(self.file_path, first.span.start, third.span.end);
        Ok(Some(Spanned::new(replacement_char, span)))
    }

    /// Reads one character after applying pass 1 (trigraphs + CRLF normalization)
    fn read_pass1_char(&mut self) -> Result<Option<Spanned<'src, char>>, SourceError<'src>> {
        let Some(ch) = self.read_raw_char()? else {
            return Ok(None);
        };

        // Trigraph replacement
        let ch = if ch.value == '?' {
            if let Some(replacement) = self.match_trigraph(ch)? {
                replacement
            } else {
                // `match_trigraph` pushed non-trigraphs to `lookahead`; read next
                return self.read_raw_char();
            }
        } else {
            ch
        };

        // CRLF normalization
        if ch.value == '\r' {
            if let Some(next) = self.read_raw_char()? {
                if next.value == '\n' {
                    // Merge "\r\n" into single "\n" with combined span
                    let span = Span::new(self.file_path, ch.span.start, next.span.end);
                    return Ok(Some(Spanned::new('\n', span)));
                }
                self.lookahead.push_back(next);
            }
        }

        Ok(Some(ch))
    }
}

impl<'src> Iterator for SourceReader<'src> {
    type Item = Result<Spanned<'src, char>, SourceError<'src>>;

    fn next(&mut self) -> Option<Self::Item> {
        // Check pending first (from pass 2 lookahead)
        if let Some(ch) = self.pending.take() {
            self.last_output_char = Some(ch.value);
            return Some(Ok(ch));
        }

        // Apply pass 2: line splicing
        loop {
            let ch = match self.read_pass1_char() {
                Ok(Some(ch)) => ch,
                Ok(None) => {
                    // Reached EOF - validate phase 2 requirements:
                    // "A source file that is not empty shall end in a new-line character"
                    if let Some(last_ch) = self.last_output_char {
                        if last_ch != '\n' {
                            return Some(Err(SourceError::MissingFinalNewline));
                        }
                    }
                    return None;
                }
                Err(e) => return Some(Err(e)),
            };

            // Check for backslash-newline (line splicing)
            if ch.value == '\\' {
                match self.read_pass1_char() {
                    Ok(Some(next)) if next.value == '\n' => {
                        // Found "\\\n" - skip both, continue loop
                        continue;
                    }
                    Ok(Some(next)) => {
                        // Not line splice - save next and return backslash
                        self.pending = Some(next);
                        self.last_output_char = Some(ch.value);
                        return Some(Ok(ch));
                    }
                    Ok(None) => {
                        // Backslash at EOF - will fail EOF check since last char isn't '\n'
                        self.last_output_char = Some(ch.value);
                        return Some(Ok(ch));
                    }
                    Err(e) => return Some(Err(e)),
                }
            }

            // Not a backslash; track and return
            self.last_output_char = Some(ch.value);
            return Some(Ok(ch));
        }
    }
}

#[cfg(test)]
mod tests;
