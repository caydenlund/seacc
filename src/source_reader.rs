use crate::span::{ByteOffset, Span, Spanned};
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Result as IoResult};
use std::path::Path;
use std::str;

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
    /// Lookahead buffer for trigraph detection (up to 3 chars)
    lookahead: VecDeque<Spanned<'src, char>>,
}

impl<'src> SourceReader<'src> {
    /// Creates a new `SourceReader` from a file path
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or read.
    pub fn new(file_path: &'src str) -> IoResult<Self> {
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
        })
    }

    /// Reads the next UTF-8 character from the buffer, refilling if necessary
    fn read_raw_char(&mut self) -> IoResult<Option<Spanned<'src, char>>> {
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
                return Ok(Some(Spanned::new('\u{FFFD}', span)));
            };

            if remaining.len() < char_len {
                // Need to read more
                if self.eof {
                    // Invalid UTF-8
                    self.pos += 1;
                    self.byte_offset += 1;
                    let end_offset = ByteOffset(self.byte_offset);
                    let span = Span::new(self.file_path, start_offset, end_offset);
                    return Ok(Some(Spanned::new('\u{FFFD}', span)));
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
            return Ok(Some(Spanned::new('\u{FFFD}', span)));
        }
    }

    /// Attempts to match and replace a trigraph sequence starting with "??"
    ///
    /// Returns the replacement character if a trigraph is found.
    /// If not a trigraph, returns the characters that should be output instead (in order).
    fn match_trigraph(
        &mut self,
        first: Spanned<'src, char>,
    ) -> IoResult<Option<Spanned<'src, char>>> {
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

    /// Fills the lookahead buffer with the next character, performing trigraph replacement
    fn fill_lookahead(&mut self) -> IoResult<()> {
        if let Some(ch) = self.read_raw_char()? {
            if ch.value == '?' {
                if let Some(r) = self.match_trigraph(ch)? {
                    self.lookahead.push_back(r);
                }
            } else {
                self.lookahead.push_back(ch);
            }
        }
        Ok(())
    }
}

impl<'src> Iterator for SourceReader<'src> {
    type Item = IoResult<Spanned<'src, char>>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.lookahead.is_empty() {
            return Some(Ok(self.lookahead.pop_front()?));
        }

        match self.fill_lookahead() {
            Ok(()) => (!self.lookahead.is_empty()).then(|| Ok(self.lookahead.pop_front().unwrap())),
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_reading() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "abc").unwrap();
        let path = file.path().to_str().unwrap();

        let reader = SourceReader::new(path).unwrap();
        let chars: Vec<char> = reader.map(|r| r.unwrap().value).collect();

        assert_eq!(chars, vec!['a', 'b', 'c']);
    }

    #[test]
    fn test_utf8_multibyte() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "a→ℝ𝕏").unwrap();
        let path = file.path().to_str().unwrap();

        let reader = SourceReader::new(path).unwrap();
        let chars: Vec<char> = reader.map(|r| r.unwrap().value).collect();

        assert_eq!(chars, vec!['a', '→', 'ℝ', '𝕏']);
    }

    #[test]
    fn test_trigraph_replacement() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "??=??/??'").unwrap();
        let path = file.path().to_str().unwrap();

        let reader = SourceReader::new(path).unwrap();
        let chars: Vec<char> = reader.map(|r| r.unwrap().value).collect();

        assert_eq!(chars, vec!['#', '\\', '^']);
    }

    #[test]
    fn test_all_trigraphs() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "??=??(??)??/??'??<??!??>??-").unwrap();
        let path = file.path().to_str().unwrap();

        let reader = SourceReader::new(path).unwrap();
        let chars: Vec<char> = reader.map(|r| r.unwrap().value).collect();

        assert_eq!(chars, vec!['#', '[', ']', '\\', '^', '{', '|', '}', '~']);
    }

    #[test]
    fn test_non_trigraph() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "??x??y").unwrap();
        let path = file.path().to_str().unwrap();

        let reader = SourceReader::new(path).unwrap();
        let chars: Vec<char> = reader.map(|r| r.unwrap().value).collect();

        assert_eq!(chars, vec!['?', '?', 'x', '?', '?', 'y']);
    }

    #[test]
    fn test_single_question_mark() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "a?b").unwrap();
        let path = file.path().to_str().unwrap();

        let reader = SourceReader::new(path).unwrap();
        let chars: Vec<char> = reader.map(|r| r.unwrap().value).collect();

        assert_eq!(chars, vec!['a', '?', 'b']);
    }

    #[test]
    fn test_span_tracking() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "a??=b").unwrap();
        let path = file.path().to_str().unwrap();

        let reader = SourceReader::new(path).unwrap();
        let chars: Vec<Spanned<'_, char>> = reader.map(|r| r.unwrap()).collect();

        assert_eq!(chars.len(), 3);
        assert_eq!(chars[0].value, 'a');
        assert_eq!(chars[0].span.start.0, 0);
        assert_eq!(chars[0].span.end.0, 1);

        assert_eq!(chars[1].value, '#'); // Trigraph replacement
        assert_eq!(chars[1].span.start.0, 1);
        assert_eq!(chars[1].span.end.0, 4); // Spans 3 bytes

        assert_eq!(chars[2].value, 'b');
        assert_eq!(chars[2].span.start.0, 4);
        assert_eq!(chars[2].span.end.0, 5);
    }

    #[test]
    fn test_span_tracking_multibyte() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "→??=ℝ").unwrap(); // → is 3 bytes, ℝ is 3 bytes
        let path = file.path().to_str().unwrap();

        let reader = SourceReader::new(path).unwrap();
        let chars: Vec<Spanned<'_, char>> = reader.map(|r| r.unwrap()).collect();

        assert_eq!(chars.len(), 3);
        assert_eq!(chars[0].value, '→');
        assert_eq!(chars[0].span.start.0, 0);
        assert_eq!(chars[0].span.end.0, 3);

        assert_eq!(chars[1].value, '#'); // Trigraph replacement
        assert_eq!(chars[1].span.start.0, 3);
        assert_eq!(chars[1].span.end.0, 6); // Spans 3 bytes

        assert_eq!(chars[2].value, 'ℝ');
        assert_eq!(chars[2].span.start.0, 6);
        assert_eq!(chars[2].span.end.0, 9);
    }
}
