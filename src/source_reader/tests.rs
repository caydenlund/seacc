use super::*;
use std::io::Cursor;

fn make_reader(input: &[u8]) -> SourceReader<'static, Cursor<Vec<u8>>> {
    SourceReader::new("<test>", Cursor::new(input.to_vec()))
}

#[test]
fn test_basic_reading() {
    let reader = make_reader(b"abc\n");
    let chars: Vec<char> = reader.map(|r| r.unwrap().value).collect();

    assert_eq!(chars, vec!['a', 'b', 'c', '\n']);
}

#[test]
fn test_utf8_multibyte() {
    let reader = make_reader("a→ℝ𝕏\n".as_bytes());
    let chars: Vec<char> = reader.map(|r| r.unwrap().value).collect();

    assert_eq!(chars, vec!['a', '→', 'ℝ', '𝕏', '\n']);
}

#[test]
fn test_trigraph_replacement() {
    let reader = make_reader(b"??=??/??'\n");
    let chars: Vec<char> = reader.map(|r| r.unwrap().value).collect();

    assert_eq!(chars, vec!['#', '\\', '^', '\n']);
}

#[test]
fn test_all_trigraphs() {
    let reader = make_reader(b"??=??(??)??/??'??<??!??>??-\n");
    let chars: Vec<char> = reader.map(|r| r.unwrap().value).collect();

    assert_eq!(
        chars,
        vec!['#', '[', ']', '\\', '^', '{', '|', '}', '~', '\n']
    );
}

#[test]
fn test_non_trigraph() {
    let reader = make_reader(b"??x??y\n");
    let chars: Vec<char> = reader.map(|r| r.unwrap().value).collect();

    assert_eq!(chars, vec!['?', '?', 'x', '?', '?', 'y', '\n']);
}

#[test]
fn test_single_question_mark() {
    let reader = make_reader(b"a?b\n");
    let chars: Vec<char> = reader.map(|r| r.unwrap().value).collect();

    assert_eq!(chars, vec!['a', '?', 'b', '\n']);
}

#[test]
fn test_span_tracking() {
    let reader = make_reader(b"a??=b\n");
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
    let reader = make_reader("→??=ℝ\n".as_bytes()); // "→" is 3 bytes; "ℝ" is 3 bytes
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
    let reader = make_reader(b"a\r\nb\rc\n");
    let chars: Vec<char> = reader.map(|r| r.unwrap().value).collect();

    // "\r\n" should become "\n"; standalone "\r" stays; "\n" stays
    assert_eq!(chars, vec!['a', '\n', 'b', '\r', 'c', '\n']);
}

#[test]
fn test_backslash_newline_removal() {
    let reader = make_reader(b"a\\\nb\\c\nd\n");
    let chars: Vec<char> = reader.map(|r| r.unwrap().value).collect();

    // "\\\n" should be removed (line splicing)
    assert_eq!(chars, vec!['a', 'b', '\\', 'c', '\n', 'd', '\n']);
}

#[test]
fn test_backslash_crlf_removal() {
    let reader = make_reader(b"a\\\r\nb\n");
    let chars: Vec<char> = reader.map(|r| r.unwrap().value).collect();

    // "\r\n" normalized to "\n" first, then "\\\n" removed
    assert_eq!(chars, vec!['a', 'b', '\n']);
}

#[test]
fn test_combined_passes() {
    let reader = make_reader(b"??=\\\n#\r\ntest\n");
    let chars: Vec<char> = reader.map(|r| r.unwrap().value).collect();

    // "??=" -> "#"; "\\\n" removed; "\r\n" -> "\n"
    assert_eq!(chars, vec!['#', '#', '\n', 't', 'e', 's', 't', '\n']);
}

#[test]
fn test_multiline_string_splicing() {
    let reader = make_reader(b"\"hel\\\nlo\"\n");
    let chars: Vec<char> = reader.map(|r| r.unwrap().value).collect();

    // Line splicing should work inside strings
    assert_eq!(chars, vec!['"', 'h', 'e', 'l', 'l', 'o', '"', '\n']);
}

#[test]
fn test_simple_line_splice() {
    let reader = make_reader(b"a\\\nb\n");
    let chars: Vec<char> = reader.map(|r| r.unwrap().value).collect();

    // "\\\n" removed
    assert_eq!(chars, vec!['a', 'b', '\n']);
}

#[test]
fn test_empty_file_allowed() {
    let reader = make_reader(b"");
    let result: Result<Vec<char>, _> = reader.map(|r| r.map(|s| s.value)).collect();

    // Empty files are allowed
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![]);
}

#[test]
fn test_file_must_end_with_newline() {
    let reader = make_reader(b"abc");
    let result: Result<Vec<char>, _> = reader.map(|r| r.map(|s| s.value)).collect();

    // Non-empty file without trailing newline should error
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        SourceError::MissingFinalNewline
    ));
}

#[test]
fn test_file_ending_with_backslash() {
    let reader = make_reader(b"abc\\");
    let result: Result<Vec<char>, _> = reader.map(|r| r.map(|s| s.value)).collect();

    // File ending with backslash (no newline) should error
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        SourceError::MissingFinalNewline
    ));
}

#[test]
fn test_file_ending_with_backslash_newline() {
    let reader = make_reader(b"abc\\\n");
    let result: Result<Vec<char>, _> = reader.map(|r| r.map(|s| s.value)).collect();

    // File ending with "\\\n" should error (line splicing removes it, leaving no final newline)
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        SourceError::MissingFinalNewline
    ));
}

#[test]
fn test_file_ending_with_newline_ok() {
    let reader = make_reader(b"abc\n");
    let result: Result<Vec<char>, _> = reader.map(|r| r.map(|s| s.value)).collect();

    // File ending with newline is OK
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!['a', 'b', 'c', '\n']);
}

#[test]
fn test_line_splice_not_at_eof_ok() {
    let reader = make_reader(b"abc\\\ndef\n");
    let result: Result<Vec<char>, _> = reader.map(|r| r.map(|s| s.value)).collect();

    // Line splice in the middle is OK, file ends with proper newline
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!['a', 'b', 'c', 'd', 'e', 'f', '\n']);
}

#[test]
fn test_invalid_utf8() {
    // Write invalid UTF-8: 0xFF is not a valid UTF-8 start byte
    let reader = make_reader(b"abc\xFFdef\n");
    let result: Result<Vec<char>, _> = reader.map(|r| r.map(|s| s.value)).collect();

    // Should error on invalid UTF-8
    assert!(result.is_err());
    match result.unwrap_err() {
        SourceError::InvalidUtf8(span) => {
            // Span should point to the invalid byte at position 3
            assert_eq!(span.start.0, 3);
            assert_eq!(span.end.0, 4);
        }
        _ => panic!("Expected InvalidUtf8 error"),
    }
}
