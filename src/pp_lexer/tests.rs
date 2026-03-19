use super::*;
use crate::source_reader::SourceReader;
use crate::token::Punctuator;
use std::io::Write;
use tempfile::NamedTempFile;

fn lex_str(input: &str) -> Result<Vec<PreprocessingToken>, String> {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "{input}").unwrap();
    let path_owned = file.path().to_str().unwrap().to_string();

    let reader = SourceReader::new(&path_owned).expect("Failed to create reader");
    let lexer = PpLexer::new(reader);

    let result: Result<Vec<_>, String> = lexer
        .map(|r| r.map(|s| s.value).map_err(|e| e.to_string()))
        .collect();
    result
}

#[test]
fn test_simple_identifier() {
    let tokens = lex_str("foo").unwrap();
    assert_eq!(tokens.len(), 1);
    match &tokens[0] {
        PreprocessingToken::Identifier(id) => {
            assert_eq!(id.as_ref(), "foo");
        }
        _ => panic!("Expected identifier"),
    }
}

#[test]
fn test_identifier_with_underscore() {
    let tokens = lex_str("_bar").unwrap();
    assert_eq!(tokens.len(), 1);
    match &tokens[0] {
        PreprocessingToken::Identifier(id) => {
            assert_eq!(id.as_ref(), "_bar");
        }
        _ => panic!("Expected identifier"),
    }
}

#[test]
fn test_identifier_with_numbers() {
    let tokens = lex_str("test123").unwrap();
    assert_eq!(tokens.len(), 1);
    match &tokens[0] {
        PreprocessingToken::Identifier(id) => {
            assert_eq!(id.as_ref(), "test123");
        }
        _ => panic!("Expected identifier"),
    }
}

#[test]
fn test_underscore_only() {
    let tokens = lex_str("_").unwrap();
    assert_eq!(tokens.len(), 1);
    match &tokens[0] {
        PreprocessingToken::Identifier(id) => {
            assert_eq!(id.as_ref(), "_");
        }
        _ => panic!("Expected identifier"),
    }
}

#[test]
fn test_single_char_punctuators() {
    let test_cases = vec![
        ("[", Punctuator::LBracket),
        ("]", Punctuator::RBracket),
        ("(", Punctuator::LParen),
        (")", Punctuator::RParen),
        ("{", Punctuator::LCurly),
        ("}", Punctuator::RCurly),
        (".", Punctuator::Dot),
        ("&", Punctuator::Amp),
        ("*", Punctuator::Asterisk),
        ("+", Punctuator::Plus),
        ("-", Punctuator::Minus),
        ("~", Punctuator::Tilde),
        ("!", Punctuator::Exclamation),
        ("/", Punctuator::Slash),
        ("%", Punctuator::Percent),
        ("<", Punctuator::Lt),
        (">", Punctuator::Gt),
        ("^", Punctuator::BitXor),
        ("|", Punctuator::BitOr),
        ("?", Punctuator::Question),
        (":", Punctuator::Colon),
        (";", Punctuator::Semicolon),
        ("=", Punctuator::Assign),
        (",", Punctuator::Comma),
        ("#", Punctuator::Hash),
    ];

    for (input, expected) in test_cases {
        let tokens = lex_str(input).unwrap();
        assert_eq!(tokens.len(), 1, "Failed for input: {input}");
        match &tokens[0] {
            PreprocessingToken::Punctuator(p) => {
                assert!(
                    std::mem::discriminant(p) == std::mem::discriminant(&expected),
                    "Failed for input: {input}, got {p:?}, expected {expected:?}"
                );
            }
            _ => panic!("Expected punctuator for input: {input}"),
        }
    }
}

#[test]
fn test_multi_char_punctuators() {
    let test_cases = vec![
        ("->", Punctuator::Arrow),
        ("++", Punctuator::Incr),
        ("--", Punctuator::Decr),
        ("<<", Punctuator::LShift),
        (">>", Punctuator::RShift),
        ("<=", Punctuator::Leq),
        (">=", Punctuator::Geq),
        ("==", Punctuator::Eq),
        ("!=", Punctuator::Neq),
        ("&&", Punctuator::And),
        ("||", Punctuator::Or),
        ("*=", Punctuator::StarAssign),
        ("/=", Punctuator::SlashAssign),
        ("%=", Punctuator::PercentAssign),
        ("+=", Punctuator::PlusAssign),
        ("-=", Punctuator::MinusAssign),
        ("&=", Punctuator::BitAndAssign),
        ("^=", Punctuator::BitXorAssign),
        ("|=", Punctuator::BitOrAssign),
        ("##", Punctuator::HashHash),
        ("<<=", Punctuator::LShiftAssign),
        (">>=", Punctuator::RShiftAssign),
        ("...", Punctuator::Ellips),
    ];

    for (input, expected) in test_cases {
        let tokens = lex_str(input).unwrap();
        assert_eq!(tokens.len(), 1, "Failed for input: {input}");
        match &tokens[0] {
            PreprocessingToken::Punctuator(p) => {
                assert!(
                    std::mem::discriminant(p) == std::mem::discriminant(&expected),
                    "Failed for input: {input}, got {p:?}, expected {expected:?}"
                );
            }
            _ => panic!("Expected punctuator for input: {input}"),
        }
    }
}

#[test]
fn test_digraphs() {
    let test_cases = vec![
        ("<:", Punctuator::LBracket),
        (":>", Punctuator::RBracket),
        ("<%", Punctuator::LCurly),
        ("%>", Punctuator::RCurly),
        ("%:", Punctuator::Hash),
        ("%:%:", Punctuator::HashHash),
    ];

    for (input, expected) in test_cases {
        let tokens = lex_str(input).unwrap();
        assert_eq!(tokens.len(), 1, "Failed for digraph: {input}");
        match &tokens[0] {
            PreprocessingToken::Punctuator(p) => {
                assert!(
                    std::mem::discriminant(p) == std::mem::discriminant(&expected),
                    "Failed for digraph: {input}, got {p:?}, expected {expected:?}"
                );
            }
            _ => panic!("Expected punctuator for digraph: {input}"),
        }
    }
}

#[test]
fn test_longest_match() {
    // Test that "<<=" is lexed as one token, not "<<" + "="
    let tokens = lex_str("<<=").unwrap();
    assert_eq!(tokens.len(), 1);
    match &tokens[0] {
        PreprocessingToken::Punctuator(Punctuator::LShiftAssign) => {}
        _ => panic!("Expected LShiftAssign"),
    }

    // Test that "++" is one token
    let tokens = lex_str("++").unwrap();
    assert_eq!(tokens.len(), 1);
    match &tokens[0] {
        PreprocessingToken::Punctuator(Punctuator::Incr) => {}
        _ => panic!("Expected Incr"),
    }
}

#[test]
fn test_whitespace_skipping() {
    let tokens = lex_str("foo  \t  bar").unwrap();
    assert_eq!(tokens.len(), 2);
    match (&tokens[0], &tokens[1]) {
        (PreprocessingToken::Identifier(id1), PreprocessingToken::Identifier(id2)) => {
            assert_eq!(id1.as_ref(), "foo");
            assert_eq!(id2.as_ref(), "bar");
        }
        _ => panic!("Expected two identifiers"),
    }
}

#[test]
fn test_line_comment() {
    let tokens = lex_str("foo // comment\nbar").unwrap();
    assert_eq!(tokens.len(), 2);
    match (&tokens[0], &tokens[1]) {
        (PreprocessingToken::Identifier(id1), PreprocessingToken::Identifier(id2)) => {
            assert_eq!(id1.as_ref(), "foo");
            assert_eq!(id2.as_ref(), "bar");
        }
        _ => panic!("Expected two identifiers"),
    }
}

#[test]
fn test_block_comment() {
    let tokens = lex_str("foo/*comment*/bar").unwrap();
    assert_eq!(tokens.len(), 2);
    match (&tokens[0], &tokens[1]) {
        (PreprocessingToken::Identifier(id1), PreprocessingToken::Identifier(id2)) => {
            assert_eq!(id1.as_ref(), "foo");
            assert_eq!(id2.as_ref(), "bar");
        }
        _ => panic!("Expected two identifiers"),
    }
}

#[test]
fn test_block_comment_multiline() {
    let tokens = lex_str("foo/*line1\nline2*/bar").unwrap();
    assert_eq!(tokens.len(), 2);
    match (&tokens[0], &tokens[1]) {
        (PreprocessingToken::Identifier(id1), PreprocessingToken::Identifier(id2)) => {
            assert_eq!(id1.as_ref(), "foo");
            assert_eq!(id2.as_ref(), "bar");
        }
        _ => panic!("Expected two identifiers"),
    }
}

#[test]
fn test_unterminated_block_comment() {
    let result = lex_str("foo /* no end");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unterminated block comment"));
}

#[test]
fn test_complex_expression() {
    let tokens = lex_str("x = y + 42;").unwrap();
    assert_eq!(tokens.len(), 6);

    match &tokens[..] {
        [
            PreprocessingToken::Identifier(id1),
            PreprocessingToken::Punctuator(Punctuator::Assign),
            PreprocessingToken::Identifier(id2),
            PreprocessingToken::Punctuator(Punctuator::Plus),
            PreprocessingToken::PpNumber(num),
            PreprocessingToken::Punctuator(Punctuator::Semicolon),
        ] => {
            assert_eq!(id1.as_ref(), "x");
            assert_eq!(id2.as_ref(), "y");
            assert_eq!(num, "42");
        }
        _ => panic!("Unexpected token sequence"),
    }
}

#[test]
fn test_digraph_in_expression() {
    // "<:0:>" should be "[", "0", "]"
    let tokens = lex_str("<:0:>").unwrap();
    assert_eq!(tokens.len(), 3);

    match &tokens[..] {
        [
            PreprocessingToken::Punctuator(Punctuator::LBracket),
            PreprocessingToken::PpNumber(num),
            PreprocessingToken::Punctuator(Punctuator::RBracket),
        ] => {
            assert_eq!(num, "0");
        }
        _ => panic!("Unexpected token sequence"),
    }
}

#[test]
fn test_adjacent_operators() {
    // "<<=" should be one token
    let tokens = lex_str("<<=").unwrap();
    assert_eq!(tokens.len(), 1);

    // "< <=" should be two tokens
    let tokens = lex_str("< <=").unwrap();
    assert_eq!(tokens.len(), 2);
}

// ============================================================================
// ERROR TESTS
// ============================================================================

#[test]
fn test_unexpected_char_at_sign() {
    let result = lex_str("foo @ bar");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character '@'"));
}

#[test]
fn test_unexpected_char_dollar() {
    let result = lex_str("$var");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character '$'"));
}

#[test]
fn test_unexpected_char_backtick() {
    let result = lex_str("`command`");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character '`'"));
}

#[test]
fn test_unexpected_char_backslash() {
    let result = lex_str("foo \\ bar");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character '\\'"));
}

#[test]
fn test_unexpected_char_in_expression() {
    let result = lex_str("x = y @ 42");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character '@'"));
}

#[test]
fn test_unexpected_char_after_valid_tokens() {
    let result = lex_str("int x; $ int y;");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character '$'"));
}

#[test]
fn test_unexpected_char_unicode_snowman() {
    let result = lex_str("foo ☃ bar");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character"));
}

#[test]
fn test_unexpected_char_unicode_emoji() {
    let result = lex_str("x = 🚀");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character"));
}

#[test]
fn test_unterminated_block_comment_simple() {
    let result = lex_str("foo /* comment without end");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unterminated block comment"));
}

#[test]
fn test_unterminated_block_comment_at_start() {
    let result = lex_str("/* no closing");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unterminated block comment"));
}

#[test]
fn test_unterminated_block_comment_multiline() {
    let result = lex_str("foo /*\n line1\n line2\n no end");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unterminated block comment"));
}

#[test]
fn test_unterminated_block_comment_with_asterisks() {
    // Has asterisks but not followed by slash
    let result = lex_str("foo /* comment * still * going");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unterminated block comment"));
}

#[test]
fn test_unterminated_block_comment_almost_closed() {
    // Has "*" at end but no "/" after it
    let result = lex_str("foo /* comment *");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unterminated block comment"));
}

#[test]
fn test_unterminated_block_comment_empty() {
    let result = lex_str("/*");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unterminated block comment"));
}

#[test]
fn test_unterminated_block_comment_after_code() {
    let result = lex_str("int x = 42; /* comment starts here");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unterminated block comment"));
}

#[test]
fn test_multiple_unexpected_chars_first_reported() {
    // Should report the first unexpected character
    let result = lex_str("@ $");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character '@'"));
}

#[test]
fn test_unexpected_char_between_valid_operators() {
    let result = lex_str("x + @ - y");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character '@'"));
}

#[test]
fn test_unexpected_char_in_comment_like_but_invalid() {
    // Starts like a comment but isn't
    let result = lex_str("/ @ comment");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character '@'"));
}

#[test]
fn test_unterminated_with_slash_not_after_star() {
    let result = lex_str("/* test / still going");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unterminated block comment"));
}

#[test]
fn test_unexpected_char_grave_accent() {
    let result = lex_str("foo`bar");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character '`'"));
}

#[test]
fn test_unexpected_char_hash_at_operator_position() {
    // '@' in a binary operator position
    let result = lex_str("a @ b");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character '@'"));
}

#[test]
fn test_unexpected_char_pound_sterling() {
    let result = lex_str("int £var = 0;");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character"));
}

#[test]
fn test_unexpected_char_euro_symbol() {
    let result = lex_str("€100");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character"));
}

#[test]
fn test_unterminated_block_comment_with_newlines_and_stars() {
    let result = lex_str("/*\n * Documentation style\n * but no end\n *");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unterminated block comment"));
}

#[test]
fn test_unterminated_block_comment_only_slash() {
    let result = lex_str("/* /");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unterminated block comment"));
}

#[test]
fn test_unexpected_char_after_number() {
    let result = lex_str("123@456");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character '@'"));
}

#[test]
fn test_unexpected_char_after_punctuator() {
    let result = lex_str("+ @ -");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character '@'"));
}

#[test]
fn test_unterminated_block_comment_many_stars() {
    let result = lex_str("/* comment ****");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unterminated block comment"));
}

#[test]
fn test_unexpected_char_section_symbol() {
    let result = lex_str("§ section");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character"));
}

#[test]
fn test_unexpected_char_middle_of_tokens() {
    let result = lex_str("hello $ world");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character '$'"));
}

#[test]
fn test_unterminated_comment_with_close_sequence_split() {
    // The "*" and "/" are not adjacent
    let result = lex_str("/* test * \n /");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unterminated block comment"));
}

#[test]
fn test_unexpected_char_japanese_character() {
    let result = lex_str("int あ = 5;");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character"));
}

#[test]
fn test_unterminated_block_comment_multiple_star_slash_attempts() {
    // Has "*/" pattern but not as adjacent characters
    let result = lex_str("/* * / * / ");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unterminated block comment"));
}

#[test]
fn test_unexpected_char_after_complete_statement() {
    let result = lex_str("int x = 0; @ return x;");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character '@'"));
}

#[test]
fn test_unexpected_char_tilde_accent() {
    let result = lex_str("señor ñ");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character"));
}

#[test]
fn test_unexpected_char_at_end_of_file() {
    let result = lex_str("int x = 42 @");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character '@'"));
}

#[test]
fn test_unexpected_char_circumflex_accent() {
    let result = lex_str("café");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character"));
}

#[test]
fn test_unterminated_block_comment_with_only_stars() {
    let result = lex_str("/* ****");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unterminated block comment"));
}

#[test]
fn test_unexpected_char_start_of_file() {
    let result = lex_str("@ int x;");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character '@'"));
}

#[test]
fn test_unexpected_char_in_pp_number_context() {
    let result = lex_str("123 @ 456");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character '@'"));
}

#[test]
fn test_unterminated_comment_star_at_eof() {
    let result = lex_str("/* comment ending with *");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unterminated block comment"));
}

#[test]
fn test_unexpected_char_chinese_character() {
    let result = lex_str("int 变量 = 5;");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character"));
}

#[test]
fn test_unexpected_char_arabic_character() {
    let result = lex_str("int متغير = 5;");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character"));
}

#[test]
fn test_unexpected_char_after_digraph() {
    let result = lex_str("<: @ :>");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character '@'"));
}

#[test]
fn test_unterminated_block_comment_reverse_closing() {
    // Has "/" followed by "*" which is not the closing sequence
    let result = lex_str("/* comment /* more");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unterminated block comment"));
}

#[test]
fn test_unexpected_char_mathematical_symbol() {
    let result = lex_str("x ≠ y");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character"));
}

#[test]
fn test_unexpected_char_cyrillic() {
    let result = lex_str("int Ф = 5;");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character"));
}

#[test]
fn test_unterminated_block_comment_with_many_slashes() {
    let result = lex_str("/* comment ////");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unterminated block comment"));
}

#[test]
fn test_unexpected_char_in_punctuator_sequence() {
    let result = lex_str("x +@ y");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character '@'"));
}

#[test]
fn test_unterminated_block_comment_star_star_slash() {
    // "**/" should still close the comment, but this one isn't closed
    let result = lex_str("/* test **");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unterminated block comment"));
}

#[test]
fn test_unexpected_char_copyright_symbol() {
    let result = lex_str("© 2024");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character"));
}

#[test]
fn test_unexpected_char_registered_trademark() {
    let result = lex_str("MyLib® v1.0");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character"));
}

#[test]
fn test_unexpected_char_degree_symbol() {
    let result = lex_str("angle = 90°");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unexpected character"));
}

#[test]
fn test_unterminated_block_comment_documentation_style_incomplete() {
    // Doc comment style but unterminated
    let result = lex_str("/**\n * @brief test\n * @param x");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unterminated block comment"));
}
