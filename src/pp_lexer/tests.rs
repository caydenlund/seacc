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
    assert_eq!(tokens.len(), 2); // "foo", newline
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
    assert_eq!(tokens.len(), 2); // "_bar", newline
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
    assert_eq!(tokens.len(), 2); // "test123", newline
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
    assert_eq!(tokens.len(), 2); // "_", newline
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
        assert_eq!(tokens.len(), 2, "Failed for input: {input}"); // punctuator, newline
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
        assert_eq!(tokens.len(), 2, "Failed for input: {input}"); // punctuator, newline
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
        assert_eq!(tokens.len(), 2, "Failed for digraph: {input}"); // digraph, newline
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
    let tokens = lex_str("<<=").unwrap();
    assert_eq!(tokens.len(), 2); // "<<=", newline
    match &tokens[0] {
        PreprocessingToken::Punctuator(Punctuator::LShiftAssign) => {}
        _ => panic!("Expected LShiftAssign"),
    }

    let tokens = lex_str("++").unwrap();
    assert_eq!(tokens.len(), 2); // "++", newline
    match &tokens[0] {
        PreprocessingToken::Punctuator(Punctuator::Incr) => {}
        _ => panic!("Expected Incr"),
    }
}

#[test]
fn test_whitespace_tokens() {
    let tokens = lex_str("foo  \t  bar").unwrap();
    assert_eq!(tokens.len(), 4); // "foo", space, "bar", newline
    match (&tokens[0], &tokens[1], &tokens[2]) {
        (
            PreprocessingToken::Identifier(id1),
            PreprocessingToken::Whitespace,
            PreprocessingToken::Identifier(id2),
        ) => {
            assert_eq!(id1.as_ref(), "foo");
            assert_eq!(id2.as_ref(), "bar");
        }
        _ => panic!("Expected identifier, whitespace, identifier"),
    }
}

#[test]
fn test_line_comment() {
    let tokens = lex_str("foo // comment\nbar").unwrap();
    // "foo", "space", comment, newline, "bar", newline
    assert_eq!(tokens.len(), 6);
    match &tokens[..] {
        [
            PreprocessingToken::Identifier(id1),
            PreprocessingToken::Whitespace, // space before comment
            PreprocessingToken::Whitespace, // comment itself
            PreprocessingToken::Newline,    // explicit newline
            PreprocessingToken::Identifier(id2),
            PreprocessingToken::Newline, // from `writeln!`
        ] => {
            assert_eq!(id1.as_ref(), "foo");
            assert_eq!(id2.as_ref(), "bar");
        }
        _ => panic!("Unexpected token sequence: {tokens:?}"),
    }
}

#[test]
fn test_block_comment() {
    let tokens = lex_str("foo/*comment*/bar").unwrap();
    // "foo", comment, "bar", newline
    assert_eq!(tokens.len(), 4);
    match &tokens[..] {
        [
            PreprocessingToken::Identifier(id1),
            PreprocessingToken::Whitespace, // comment
            PreprocessingToken::Identifier(id2),
            PreprocessingToken::Newline, // from `writeln!`
        ] => {
            assert_eq!(id1.as_ref(), "foo");
            assert_eq!(id2.as_ref(), "bar");
        }
        _ => panic!("Unexpected token sequence: {tokens:?}"),
    }
}

#[test]
fn test_block_comment_multiline() {
    let tokens = lex_str("foo/*line1\nline2*/bar").unwrap();
    // "foo", comment, "bar", newline
    assert_eq!(tokens.len(), 4);
    match &tokens[..] {
        [
            PreprocessingToken::Identifier(id1),
            PreprocessingToken::Whitespace, // comment (with embedded newline)
            PreprocessingToken::Identifier(id2),
            PreprocessingToken::Newline, // from `writeln!`
        ] => {
            assert_eq!(id1.as_ref(), "foo");
            assert_eq!(id2.as_ref(), "bar");
        }
        _ => panic!("Unexpected token sequence: {tokens:?}"),
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
    // "x", space, "=", space, "y", space, "+", space, "42", ";", newline
    assert_eq!(tokens.len(), 11);

    match &tokens[..] {
        [
            PreprocessingToken::Identifier(id1),
            PreprocessingToken::Whitespace,
            PreprocessingToken::Punctuator(Punctuator::Assign),
            PreprocessingToken::Whitespace,
            PreprocessingToken::Identifier(id2),
            PreprocessingToken::Whitespace,
            PreprocessingToken::Punctuator(Punctuator::Plus),
            PreprocessingToken::Whitespace,
            PreprocessingToken::PpNumber(num),
            PreprocessingToken::Punctuator(Punctuator::Semicolon),
            PreprocessingToken::Newline,
        ] => {
            assert_eq!(id1.as_ref(), "x");
            assert_eq!(id2.as_ref(), "y");
            assert_eq!(num, "42");
        }
        _ => panic!("Unexpected token sequence: {tokens:?}"),
    }
}

#[test]
fn test_digraph_in_expression() {
    let tokens = lex_str("<:0:>").unwrap();
    assert_eq!(tokens.len(), 4); // "[", "0", "]", newline

    match &tokens[..] {
        [
            PreprocessingToken::Punctuator(Punctuator::LBracket),
            PreprocessingToken::PpNumber(num),
            PreprocessingToken::Punctuator(Punctuator::RBracket),
            PreprocessingToken::Newline,
        ] => {
            assert_eq!(num, "0");
        }
        _ => panic!("Unexpected token sequence: {tokens:?}"),
    }
}

#[test]
fn test_adjacent_operators() {
    // "<<=", newline
    let tokens = lex_str("<<=").unwrap();
    assert_eq!(tokens.len(), 2);
    match &tokens[..] {
        [
            PreprocessingToken::Punctuator(Punctuator::LShiftAssign),
            PreprocessingToken::Newline,
        ] => {}
        _ => panic!("Unexpected tokens: {tokens:?}"),
    }

    // "<", space, "<=", newline
    let tokens = lex_str("< <=").unwrap();
    assert_eq!(tokens.len(), 4);
    match &tokens[..] {
        [
            PreprocessingToken::Punctuator(Punctuator::Lt),
            PreprocessingToken::Whitespace,
            PreprocessingToken::Punctuator(Punctuator::Leq),
            PreprocessingToken::Newline,
        ] => {}
        _ => panic!("Unexpected tokens: {tokens:?}"),
    }
}

#[test]
fn test_other_char_at_sign() {
    let tokens = lex_str("foo @ bar").unwrap();
    // "foo", space, "@", space, "bar", newline
    assert_eq!(tokens.len(), 6);
    match &tokens[2] {
        PreprocessingToken::OtherChar('@') => {}
        _ => panic!("Expected OtherChar('@'), got {:?}", tokens[2]),
    }
}

#[test]
fn test_other_char_dollar() {
    let tokens = lex_str("$var").unwrap();
    // "$", "var", newline
    assert_eq!(tokens.len(), 3);
    match &tokens[0] {
        PreprocessingToken::OtherChar('$') => {}
        _ => panic!("Expected OtherChar('$'), got {:?}", tokens[0]),
    }
}

#[test]
fn test_other_char_backtick() {
    let tokens = lex_str("`command`").unwrap();
    // "`", "command", "`", newline
    assert_eq!(tokens.len(), 4);
    match (&tokens[0], &tokens[2]) {
        (PreprocessingToken::OtherChar('`'), PreprocessingToken::OtherChar('`')) => {}
        _ => panic!("Expected backticks as OtherChar, got {tokens:?}"),
    }
}

#[test]
fn test_other_char_backslash() {
    let tokens = lex_str("foo \\ bar").unwrap();
    // "foo", space, "\", space, "bar", newline
    assert_eq!(tokens.len(), 6);
    match &tokens[2] {
        PreprocessingToken::OtherChar('\\') => {}
        _ => panic!("Expected OtherChar('\\'), got {:?}", tokens[2]),
    }
}

#[test]
fn test_other_char_in_expression() {
    let tokens = lex_str("x = y @ 42").unwrap();
    // Contains "@" as `OtherChar`
    match tokens
        .iter()
        .find(|t| matches!(t, PreprocessingToken::OtherChar('@')))
    {
        Some(_) => {}
        None => panic!("Expected OtherChar('@') in tokens: {tokens:?}"),
    }
}

#[test]
fn test_other_char_after_valid_tokens() {
    let tokens = lex_str("int x; $ int y;").unwrap();
    // Contains "$" as `OtherChar`
    match tokens
        .iter()
        .find(|t| matches!(t, PreprocessingToken::OtherChar('$')))
    {
        Some(_) => {}
        None => panic!("Expected OtherChar('$') in tokens: {tokens:?}"),
    }
}

#[test]
fn test_other_char_unicode_snowman() {
    let tokens = lex_str("foo ☃ bar").unwrap();
    // Contains snowman as `OtherChar`
    match tokens
        .iter()
        .find(|t| matches!(t, PreprocessingToken::OtherChar('☃')))
    {
        Some(_) => {}
        None => panic!("Expected OtherChar('☃') in tokens: {tokens:?}"),
    }
}

#[test]
fn test_other_char_unicode_emoji() {
    let tokens = lex_str("x = 🚀").unwrap();
    // Contains rocket emoji as OtherChar
    match tokens
        .iter()
        .find(|t| matches!(t, PreprocessingToken::OtherChar('🚀')))
    {
        Some(_) => {}
        None => panic!("Expected OtherChar('🚀') in tokens: {tokens:?}"),
    }
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
fn test_multiple_other_chars() {
    // Both @ and $ become OtherChar tokens
    let tokens = lex_str("@ $").unwrap();
    // "@", space, "$", newline
    assert_eq!(tokens.len(), 4);
    match (&tokens[0], &tokens[2]) {
        (PreprocessingToken::OtherChar('@'), PreprocessingToken::OtherChar('$')) => {}
        _ => panic!("Expected OtherChar('@') and OtherChar('$'), got {tokens:?}"),
    }
}

#[test]
fn test_other_char_between_valid_operators() {
    let tokens = lex_str("x + @ - y").unwrap();
    // Contains "@" as `OtherChar`
    match tokens
        .iter()
        .find(|t| matches!(t, PreprocessingToken::OtherChar('@')))
    {
        Some(_) => {}
        None => panic!("Expected OtherChar('@') in tokens: {tokens:?}"),
    }
}

#[test]
fn test_other_char_after_slash() {
    let tokens = lex_str("/@").unwrap();
    assert_eq!(tokens.len(), 3); // "/", "@", newline
    match (&tokens[0], &tokens[1]) {
        (
            PreprocessingToken::Punctuator(Punctuator::Slash),
            PreprocessingToken::OtherChar('@'),
        ) => {}
        _ => panic!("Expected Slash then OtherChar('@'), got {tokens:?}"),
    }
}

#[test]
fn test_unterminated_with_slash_not_after_star() {
    let result = lex_str("/* test / still going");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unterminated block comment"));
}

#[test]
fn test_other_char_grave_accent() {
    let tokens = lex_str("foo`bar").unwrap();
    // "foo", "`", "bar", newline
    match tokens
        .iter()
        .find(|t| matches!(t, PreprocessingToken::OtherChar('`')))
    {
        Some(_) => {}
        None => panic!("Expected OtherChar('`') in tokens: {tokens:?}"),
    }
}

#[test]
fn test_other_char_at_operator_position() {
    // "@" in a binary operator position
    let tokens = lex_str("a @ b").unwrap();
    match tokens
        .iter()
        .find(|t| matches!(t, PreprocessingToken::OtherChar('@')))
    {
        Some(_) => {}
        None => panic!("Expected OtherChar('@') in tokens: {tokens:?}"),
    }
}

#[test]
fn test_other_char_pound_sterling() {
    let tokens = lex_str("int £var = 0;").unwrap();
    match tokens
        .iter()
        .find(|t| matches!(t, PreprocessingToken::OtherChar('£')))
    {
        Some(_) => {}
        None => panic!("Expected OtherChar('£') in tokens: {tokens:?}"),
    }
}

#[test]
fn test_other_char_euro_symbol() {
    let tokens = lex_str("€100").unwrap();
    // "€", "100", newline
    match tokens
        .iter()
        .find(|t| matches!(t, PreprocessingToken::OtherChar('€')))
    {
        Some(_) => {}
        None => panic!("Expected OtherChar('€') in tokens: {tokens:?}"),
    }
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
fn test_other_char_after_number() {
    let tokens = lex_str("123@456").unwrap();
    // "123", "@", "456", newline
    match tokens
        .iter()
        .find(|t| matches!(t, PreprocessingToken::OtherChar('@')))
    {
        Some(_) => {}
        None => panic!("Expected OtherChar('@') in tokens: {tokens:?}"),
    }
}

#[test]
fn test_other_char_after_punctuator() {
    let tokens = lex_str("+ @ -").unwrap();
    match tokens
        .iter()
        .find(|t| matches!(t, PreprocessingToken::OtherChar('@')))
    {
        Some(_) => {}
        None => panic!("Expected OtherChar('@') in tokens: {tokens:?}"),
    }
}

#[test]
fn test_unterminated_block_comment_many_stars() {
    let result = lex_str("/* comment ****");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unterminated block comment"));
}

#[test]
fn test_other_char_section_symbol() {
    let tokens = lex_str("§ section").unwrap();
    match tokens
        .iter()
        .find(|t| matches!(t, PreprocessingToken::OtherChar('§')))
    {
        Some(_) => {}
        None => panic!("Expected OtherChar('§') in tokens: {tokens:?}"),
    }
}

#[test]
fn test_other_char_middle_of_tokens() {
    let tokens = lex_str("hello $ world").unwrap();
    match tokens
        .iter()
        .find(|t| matches!(t, PreprocessingToken::OtherChar('$')))
    {
        Some(_) => {}
        None => panic!("Expected OtherChar('$') in tokens: {tokens:?}"),
    }
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
fn test_other_char_japanese_character() {
    let tokens = lex_str("int あ = 5;").unwrap();
    match tokens
        .iter()
        .find(|t| matches!(t, PreprocessingToken::OtherChar('あ')))
    {
        Some(_) => {}
        None => panic!("Expected OtherChar('あ') in tokens: {tokens:?}"),
    }
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
fn test_other_char_after_complete_statement() {
    let tokens = lex_str("int x = 0; @ return x;").unwrap();
    match tokens
        .iter()
        .find(|t| matches!(t, PreprocessingToken::OtherChar('@')))
    {
        Some(_) => {}
        None => panic!("Expected OtherChar('@') in tokens: {tokens:?}"),
    }
}

#[test]
fn test_other_char_tilde_accent() {
    let tokens = lex_str("señor ñ").unwrap();
    // Contains "ñ" as `OtherChar`
    match tokens
        .iter()
        .find(|t| matches!(t, PreprocessingToken::OtherChar('ñ')))
    {
        Some(_) => {}
        None => panic!("Expected OtherChar('ñ') in tokens: {tokens:?}"),
    }
}

#[test]
fn test_other_char_at_end_of_file() {
    let tokens = lex_str("int x = 42 @").unwrap();
    match tokens
        .iter()
        .find(|t| matches!(t, PreprocessingToken::OtherChar('@')))
    {
        Some(_) => {}
        None => panic!("Expected OtherChar('@') in tokens: {tokens:?}"),
    }
}

#[test]
fn test_other_char_circumflex_accent() {
    let tokens = lex_str("café").unwrap();
    // Contains "é" as `OtherChar`
    match tokens
        .iter()
        .find(|t| matches!(t, PreprocessingToken::OtherChar('é')))
    {
        Some(_) => {}
        None => panic!("Expected OtherChar('é') in tokens: {tokens:?}"),
    }
}

#[test]
fn test_unterminated_block_comment_with_only_stars() {
    let result = lex_str("/* ****");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unterminated block comment"));
}

#[test]
fn test_other_char_start_of_file() {
    let tokens = lex_str("@ int x;").unwrap();
    match tokens
        .iter()
        .find(|t| matches!(t, PreprocessingToken::OtherChar('@')))
    {
        Some(_) => {}
        None => panic!("Expected OtherChar('@') in tokens: {tokens:?}"),
    }
}

#[test]
fn test_other_char_in_pp_number_context() {
    let tokens = lex_str("123 @ 456").unwrap();
    match tokens
        .iter()
        .find(|t| matches!(t, PreprocessingToken::OtherChar('@')))
    {
        Some(_) => {}
        None => panic!("Expected OtherChar('@') in tokens: {tokens:?}"),
    }
}

#[test]
fn test_unterminated_comment_star_at_eof() {
    let result = lex_str("/* comment ending with *");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unterminated block comment"));
}

#[test]
fn test_other_char_chinese_character() {
    let tokens = lex_str("int 变量 = 5;").unwrap();
    // Contains Chinese characters as `OtherChar` tokens
    let has_chinese = tokens
        .iter()
        .any(|t| matches!(t, PreprocessingToken::OtherChar(c) if *c == '变' || *c == '量'));
    assert!(
        has_chinese,
        "Expected Chinese characters as OtherChar in tokens: {tokens:?}"
    );
}

#[test]
fn test_other_char_arabic_character() {
    let tokens = lex_str("int متغير = 5;").unwrap();
    // Contains Arabic characters as `OtherChar` tokens
    let has_arabic = tokens
        .iter()
        .any(|t| matches!(t, PreprocessingToken::OtherChar(c) if *c >= 'ا' && *c <= 'ي'));
    assert!(
        has_arabic,
        "Expected Arabic characters as OtherChar in tokens: {tokens:?}"
    );
}

#[test]
fn test_other_char_after_digraph() {
    let tokens = lex_str("<: @ :>").unwrap();
    match tokens
        .iter()
        .find(|t| matches!(t, PreprocessingToken::OtherChar('@')))
    {
        Some(_) => {}
        None => panic!("Expected OtherChar('@') in tokens: {tokens:?}"),
    }
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
fn test_other_char_mathematical_symbol() {
    let tokens = lex_str("x ≠ y").unwrap();
    match tokens
        .iter()
        .find(|t| matches!(t, PreprocessingToken::OtherChar('≠')))
    {
        Some(_) => {}
        None => panic!("Expected OtherChar('≠') in tokens: {tokens:?}"),
    }
}

#[test]
fn test_other_char_cyrillic() {
    let tokens = lex_str("int Ф = 5;").unwrap();
    match tokens
        .iter()
        .find(|t| matches!(t, PreprocessingToken::OtherChar('Ф')))
    {
        Some(_) => {}
        None => panic!("Expected OtherChar('Ф') in tokens: {tokens:?}"),
    }
}

#[test]
fn test_unterminated_block_comment_with_many_slashes() {
    let result = lex_str("/* comment ////");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unterminated block comment"));
}

#[test]
fn test_other_char_in_punctuator_sequence() {
    let tokens = lex_str("x +@ y").unwrap();
    match tokens
        .iter()
        .find(|t| matches!(t, PreprocessingToken::OtherChar('@')))
    {
        Some(_) => {}
        None => panic!("Expected OtherChar('@') in tokens: {tokens:?}"),
    }
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
fn test_other_char_copyright_symbol() {
    let tokens = lex_str("© 2024").unwrap();
    match tokens
        .iter()
        .find(|t| matches!(t, PreprocessingToken::OtherChar('©')))
    {
        Some(_) => {}
        None => panic!("Expected OtherChar('©') in tokens: {tokens:?}"),
    }
}

#[test]
fn test_other_char_registered_trademark() {
    let tokens = lex_str("MyLib® v1.0").unwrap();
    match tokens
        .iter()
        .find(|t| matches!(t, PreprocessingToken::OtherChar('®')))
    {
        Some(_) => {}
        None => panic!("Expected OtherChar('®') in tokens: {tokens:?}"),
    }
}

#[test]
fn test_other_char_degree_symbol() {
    let tokens = lex_str("angle = 90°").unwrap();
    match tokens
        .iter()
        .find(|t| matches!(t, PreprocessingToken::OtherChar('°')))
    {
        Some(_) => {}
        None => panic!("Expected OtherChar('°') in tokens: {tokens:?}"),
    }
}

#[test]
fn test_unterminated_block_comment_documentation_style_incomplete() {
    // Doc comment style but unterminated
    let result = lex_str("/**\n * @brief test\n * @param x");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unterminated block comment"));
}

#[test]
fn test_single_space_whitespace() {
    let tokens = lex_str("a b").unwrap();
    // "a", space, "b", newline
    assert_eq!(tokens.len(), 4);
    match &tokens[1] {
        PreprocessingToken::Whitespace => {}
        _ => panic!("Expected Whitespace token, got {:?}", tokens[1]),
    }
}

#[test]
fn test_multiple_spaces_collapsed() {
    let tokens = lex_str("a     b").unwrap();
    // "a", space, "b", newline
    assert_eq!(tokens.len(), 4);
    match &tokens[1] {
        PreprocessingToken::Whitespace => {}
        _ => panic!(
            "Expected single Whitespace token for multiple spaces, got {:?}",
            tokens[1]
        ),
    }
}

#[test]
fn test_tabs_collapsed() {
    let tokens = lex_str("a\t\t\tb").unwrap();
    // "a", space, "b", newline
    assert_eq!(tokens.len(), 4);
    match &tokens[1] {
        PreprocessingToken::Whitespace => {}
        _ => panic!(
            "Expected single Whitespace token for multiple tabs, got {:?}",
            tokens[1]
        ),
    }
}

#[test]
fn test_mixed_whitespace_collapsed() {
    let tokens = lex_str("a  \t  \t  b").unwrap();
    // "a", space, "b", newline
    assert_eq!(tokens.len(), 4);
    match &tokens[1] {
        PreprocessingToken::Whitespace => {}
        _ => panic!(
            "Expected single Whitespace token for mixed whitespace, got {:?}",
            tokens[1]
        ),
    }
}

#[test]
fn test_newline_token() {
    let tokens = lex_str("a\nb").unwrap();
    // "a", newline, "b", newline
    assert_eq!(tokens.len(), 4);
    match (&tokens[1], &tokens[3]) {
        (PreprocessingToken::Newline, PreprocessingToken::Newline) => {}
        _ => panic!("Expected Newline tokens, got {tokens:?}"),
    }
}

#[test]
fn test_whitespace_not_newline() {
    let tokens = lex_str("a \nb").unwrap();
    // "a", space, newline, "b", newline
    assert_eq!(tokens.len(), 5);
    match (&tokens[1], &tokens[2]) {
        (PreprocessingToken::Whitespace, PreprocessingToken::Newline) => {}
        _ => panic!("Expected Whitespace then Newline, got {tokens:?}"),
    }
}

#[test]
fn test_vertical_tab_as_whitespace() {
    let tokens = lex_str("a\x0Bb").unwrap();
    // "a", space, "b", newline
    assert_eq!(tokens.len(), 4);
    match &tokens[1] {
        PreprocessingToken::Whitespace => {}
        _ => panic!("Expected Whitespace for vertical tab, got {:?}", tokens[1]),
    }
}

#[test]
fn test_form_feed_as_whitespace() {
    let tokens = lex_str("a\x0Cb").unwrap();
    // "a", space, "b", newline
    assert_eq!(tokens.len(), 4);
    match &tokens[1] {
        PreprocessingToken::Whitespace => {}
        _ => panic!("Expected Whitespace for form feed, got {:?}", tokens[1]),
    }
}

#[test]
fn test_multiple_newlines() {
    let tokens = lex_str("\n\n\n").unwrap();
    // 3 explicit newlines + 1 from `writeln!` = 4 newlines
    assert_eq!(tokens.len(), 4);
    for (i, token) in tokens.iter().enumerate() {
        match token {
            PreprocessingToken::Newline => {}
            _ => panic!("Expected all Newline tokens, but token {i} is {token:?}"),
        }
    }
}

#[test]
fn test_whitespace_at_start() {
    let tokens = lex_str("   foo").unwrap();
    // space, "foo", newline
    assert_eq!(tokens.len(), 3);
    match &tokens[0] {
        PreprocessingToken::Whitespace => {}
        _ => panic!("Expected Whitespace at start, got {:?}", tokens[0]),
    }
}

#[test]
fn test_whitespace_at_end() {
    let tokens = lex_str("foo   ").unwrap();
    // "foo", space, newline
    assert_eq!(tokens.len(), 3);
    match &tokens[1] {
        PreprocessingToken::Whitespace => {}
        _ => panic!("Expected Whitespace at end, got {:?}", tokens[1]),
    }
}

fn assert_pp_number(input: &str, expected: &str) {
    let tokens = lex_str(input).unwrap();
    match &tokens[0] {
        PreprocessingToken::PpNumber(n) => assert_eq!(n, expected, "input: {input:?}"),
        t => panic!("Expected PpNumber({expected:?}), got {t:?} for input {input:?}"),
    }
}

#[test]
fn test_pp_number_integer() {
    assert_pp_number("42", "42");
    assert_pp_number("0", "0");
    assert_pp_number("1234567890", "1234567890");
}

#[test]
fn test_pp_number_hex() {
    assert_pp_number("0xDEAD", "0xDEAD");
    assert_pp_number("0xFF", "0xFF");
    assert_pp_number("0x1a2b3c", "0x1a2b3c");
}

#[test]
fn test_pp_number_decimal_float() {
    assert_pp_number("3.14", "3.14");
    assert_pp_number("0.5", "0.5");
    assert_pp_number("1.", "1.");
}

#[test]
fn test_pp_number_dot_digit_start() {
    assert_pp_number(".5", ".5");
    assert_pp_number(".0", ".0");
    assert_pp_number(".123", ".123");
}

#[test]
fn test_pp_number_exponent() {
    assert_pp_number("1e10", "1e10");
    assert_pp_number("1E10", "1E10");
    assert_pp_number("1.5e10", "1.5e10");
}

#[test]
fn test_pp_number_exponent_with_sign() {
    assert_pp_number("1e+10", "1e+10");
    assert_pp_number("1e-10", "1e-10");
    assert_pp_number("1.5E+3", "1.5E+3");
    assert_pp_number("1.5E-3", "1.5E-3");
}

#[test]
fn test_pp_number_binary_exponent() {
    assert_pp_number("0x1p10", "0x1p10");
    assert_pp_number("0x1P10", "0x1P10");
    assert_pp_number("0x1.8p+1", "0x1.8p+1");
    assert_pp_number("0x1.8P-2", "0x1.8P-2");
}

#[test]
fn test_pp_number_with_suffix() {
    assert_pp_number("42u", "42u");
    assert_pp_number("42UL", "42UL");
    assert_pp_number("42ll", "42ll");
    assert_pp_number("1.5f", "1.5f");
    assert_pp_number("1.5F", "1.5F");
    assert_pp_number("1.5L", "1.5L");
}

#[test]
fn test_pp_number_dot_not_followed_by_digit() {
    // A lone "." should be a `Dot` punctuator, not a pp-number
    let tokens = lex_str(".").unwrap();
    match &tokens[0] {
        PreprocessingToken::Punctuator(Punctuator::Dot) => {}
        t => panic!("Expected Dot punctuator, got {t:?}"),
    }
}

#[test]
fn test_pp_number_dot_followed_by_identifier() {
    let tokens = lex_str(".foo").unwrap();
    assert_eq!(tokens.len(), 3); // ".", "foo", newline
    match (&tokens[0], &tokens[1]) {
        (PreprocessingToken::Punctuator(Punctuator::Dot), PreprocessingToken::Identifier(_)) => {}
        _ => panic!("Expected Dot then Identifier, got {tokens:?}"),
    }
}

#[test]
fn test_pp_number_stops_at_non_number_char() {
    let tokens = lex_str("42+").unwrap();
    assert_eq!(tokens.len(), 3); // "42", "+", newline
    match (&tokens[0], &tokens[1]) {
        (PreprocessingToken::PpNumber(n), PreprocessingToken::Punctuator(Punctuator::Plus)) => {
            assert_eq!(n, "42");
        }
        _ => panic!("Expected PpNumber then Plus, got {tokens:?}"),
    }
}
