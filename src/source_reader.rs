use crate::span::{ByteOffset, Span, Spanned};
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Error as IoError, ErrorKind, Result as IoResult};
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
            pending: None,
            last_output_char: None,
        })
    }

    /// Reads the next UTF-8 character from the buffer, refilling if necessary
    /// Checks `lookahead` first before reading from the file
    fn read_raw_char(&mut self) -> IoResult<Option<Spanned<'src, char>>> {
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

    /// Reads one character after applying pass 1 (trigraphs + CRLF normalization)
    fn read_pass1_char(&mut self) -> IoResult<Option<Spanned<'src, char>>> {
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
    type Item = IoResult<Spanned<'src, char>>;

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
                            return Some(Err(IoError::new(
                                ErrorKind::InvalidData,
                                "source file must end with a newline character",
                            )));
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
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_reading() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "abc").unwrap();
        let path = file.path().to_str().unwrap();

        let reader = SourceReader::new(path).unwrap();
        let chars: Vec<char> = reader.map(|r| r.unwrap().value).collect();

        assert_eq!(chars, vec!['a', 'b', 'c', '\n']);
    }

    #[test]
    fn test_utf8_multibyte() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "a→ℝ𝕏").unwrap();
        let path = file.path().to_str().unwrap();

        let reader = SourceReader::new(path).unwrap();
        let chars: Vec<char> = reader.map(|r| r.unwrap().value).collect();

        assert_eq!(chars, vec!['a', '→', 'ℝ', '𝕏', '\n']);
    }

    #[test]
    fn test_trigraph_replacement() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "??=??/??'").unwrap();
        let path = file.path().to_str().unwrap();

        let reader = SourceReader::new(path).unwrap();
        let chars: Vec<char> = reader.map(|r| r.unwrap().value).collect();

        assert_eq!(chars, vec!['#', '\\', '^', '\n']);
    }

    #[test]
    fn test_all_trigraphs() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "??=??(??)??/??'??<??!??>??-").unwrap();
        let path = file.path().to_str().unwrap();

        let reader = SourceReader::new(path).unwrap();
        let chars: Vec<char> = reader.map(|r| r.unwrap().value).collect();

        assert_eq!(
            chars,
            vec!['#', '[', ']', '\\', '^', '{', '|', '}', '~', '\n']
        );
    }

    #[test]
    fn test_non_trigraph() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "??x??y").unwrap();
        let path = file.path().to_str().unwrap();

        let reader = SourceReader::new(path).unwrap();
        let chars: Vec<char> = reader.map(|r| r.unwrap().value).collect();

        assert_eq!(chars, vec!['?', '?', 'x', '?', '?', 'y', '\n']);
    }

    #[test]
    fn test_single_question_mark() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "a?b").unwrap();
        let path = file.path().to_str().unwrap();

        let reader = SourceReader::new(path).unwrap();
        let chars: Vec<char> = reader.map(|r| r.unwrap().value).collect();

        assert_eq!(chars, vec!['a', '?', 'b', '\n']);
    }

    #[test]
    fn test_span_tracking() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "a??=b").unwrap();
        let path = file.path().to_str().unwrap();

        let reader = SourceReader::new(path).unwrap();
        let chars: Vec<Spanned<'_, char>> = reader.map(|r| r.unwrap()).collect();

        assert_eq!(chars.len(), 4);
        assert_eq!(chars[0].value, 'a');
        assert_eq!(chars[0].span.start.0, 0);
        assert_eq!(chars[0].span.end.0, 1);

        assert_eq!(chars[1].value, '#'); // Trigraph replacement
        assert_eq!(chars[1].span.start.0, 1);
        assert_eq!(chars[1].span.end.0, 4); // Spans 3 bytes

        assert_eq!(chars[2].value, 'b');
        assert_eq!(chars[2].span.start.0, 4);
        assert_eq!(chars[2].span.end.0, 5);

        assert_eq!(chars[3].value, '\n');
        assert_eq!(chars[3].span.start.0, 5);
        assert_eq!(chars[3].span.end.0, 6);
    }

    #[test]
    fn test_span_tracking_multibyte() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "→??=ℝ").unwrap(); // "→" is 3 bytes; "ℝ" is 3 bytes
        let path = file.path().to_str().unwrap();

        let reader = SourceReader::new(path).unwrap();
        let chars: Vec<Spanned<'_, char>> = reader.map(|r| r.unwrap()).collect();

        assert_eq!(chars.len(), 4);
        assert_eq!(chars[0].value, '→');
        assert_eq!(chars[0].span.start.0, 0);
        assert_eq!(chars[0].span.end.0, 3);

        assert_eq!(chars[1].value, '#'); // Trigraph replacement
        assert_eq!(chars[1].span.start.0, 3);
        assert_eq!(chars[1].span.end.0, 6); // Spans 3 bytes

        assert_eq!(chars[2].value, 'ℝ');
        assert_eq!(chars[2].span.start.0, 6);
        assert_eq!(chars[2].span.end.0, 9);

        assert_eq!(chars[3].value, '\n');
        assert_eq!(chars[3].span.start.0, 9);
        assert_eq!(chars[3].span.end.0, 10);
    }

    #[test]
    fn test_crlf_normalization() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "a\r\nb\rc\n").unwrap();
        let path = file.path().to_str().unwrap();

        let reader = SourceReader::new(path).unwrap();
        let chars: Vec<char> = reader.map(|r| r.unwrap().value).collect();

        // "\r\n" should become "\n"; standalone "\r" stays; "\n" stays
        assert_eq!(chars, vec!['a', '\n', 'b', '\r', 'c', '\n']);
    }

    #[test]
    fn test_backslash_newline_removal() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "a\\\nb\\c\nd\n").unwrap();
        let path = file.path().to_str().unwrap();

        let reader = SourceReader::new(path).unwrap();
        let chars: Vec<char> = reader.map(|r| r.unwrap().value).collect();

        // "\\\n" should be removed (line splicing)
        assert_eq!(chars, vec!['a', 'b', '\\', 'c', '\n', 'd', '\n']);
    }

    #[test]
    fn test_backslash_crlf_removal() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "a\\\r\nb\n").unwrap();
        let path = file.path().to_str().unwrap();

        let reader = SourceReader::new(path).unwrap();
        let chars: Vec<char> = reader.map(|r| r.unwrap().value).collect();

        // "\r\n" normalized to "\n" first, then "\\\n" removed
        assert_eq!(chars, vec!['a', 'b', '\n']);
    }

    #[test]
    fn test_combined_passes() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "??=\\\n#\r\ntest\n").unwrap();
        let path = file.path().to_str().unwrap();

        let reader = SourceReader::new(path).unwrap();
        let chars: Vec<char> = reader.map(|r| r.unwrap().value).collect();

        // "??=" -> "#"; "\\\n" removed; "\r\n" -> "\n"
        assert_eq!(chars, vec!['#', '#', '\n', 't', 'e', 's', 't', '\n']);
    }

    #[test]
    fn test_multiline_string_splicing() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "\"hel\\\nlo\"\n").unwrap();
        let path = file.path().to_str().unwrap();

        let reader = SourceReader::new(path).unwrap();
        let chars: Vec<char> = reader.map(|r| r.unwrap().value).collect();

        // Line splicing should work inside strings
        assert_eq!(chars, vec!['"', 'h', 'e', 'l', 'l', 'o', '"', '\n']);
    }

    #[test]
    fn test_simple_line_splice() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "a\\\nb\n").unwrap();
        let path = file.path().to_str().unwrap();

        let reader = SourceReader::new(path).unwrap();
        let chars: Vec<char> = reader.map(|r| r.unwrap().value).collect();

        // "\\\n" removed
        assert_eq!(chars, vec!['a', 'b', '\n']);
    }

    #[test]
    fn test_empty_file_allowed() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_str().unwrap();

        let reader = SourceReader::new(path).unwrap();
        let result: Result<Vec<char>, _> = reader.map(|r| r.map(|s| s.value)).collect();

        // Empty files are allowed
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![]);
    }

    #[test]
    fn test_file_must_end_with_newline() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "abc").unwrap();
        let path = file.path().to_str().unwrap();

        let reader = SourceReader::new(path).unwrap();
        let result: Result<Vec<char>, _> = reader.map(|r| r.map(|s| s.value)).collect();

        // Non-empty file without trailing newline should error
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidData);
    }

    #[test]
    fn test_file_ending_with_backslash() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "abc\\").unwrap();
        let path = file.path().to_str().unwrap();

        let reader = SourceReader::new(path).unwrap();
        let result: Result<Vec<char>, _> = reader.map(|r| r.map(|s| s.value)).collect();

        // File ending with backslash (no newline) should error
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidData);
    }

    #[test]
    fn test_file_ending_with_backslash_newline() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "abc\\").unwrap();
        let path = file.path().to_str().unwrap();

        let reader = SourceReader::new(path).unwrap();
        let result: Result<Vec<char>, _> = reader.map(|r| r.map(|s| s.value)).collect();

        // File ending with "\\\n" should error (line splicing removes it, leaving no final newline)
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidData);
        assert!(err.to_string().contains("newline"));
    }

    #[test]
    fn test_file_ending_with_newline_ok() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "abc").unwrap();
        let path = file.path().to_str().unwrap();

        let reader = SourceReader::new(path).unwrap();
        let result: Result<Vec<char>, _> = reader.map(|r| r.map(|s| s.value)).collect();

        // File ending with newline is OK
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!['a', 'b', 'c', '\n']);
    }

    #[test]
    fn test_line_splice_not_at_eof_ok() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "abc\\\ndef\n").unwrap();
        let path = file.path().to_str().unwrap();

        let reader = SourceReader::new(path).unwrap();
        let result: Result<Vec<char>, _> = reader.map(|r| r.map(|s| s.value)).collect();

        // Line splice in the middle is OK, file ends with proper newline
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!['a', 'b', 'c', 'd', 'e', 'f', '\n']);
    }
}
