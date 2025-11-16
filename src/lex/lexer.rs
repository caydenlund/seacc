//! lex/lexer: Definition of the main [`Lexer`] construct and logic

use super::{Dfa, LexError, RegexComponent, RegexPattern, Token, TokenType};

/// A lexical analyzer that uses a DFA to tokenize C source code
#[derive(Debug, Clone)]
pub struct Lexer {
    /// The minimized DFA used for tokenization
    dfa: Dfa,
}

impl Lexer {
    /// Constructs a new [`Lexer`] with a minimized DFA for C language tokens
    #[must_use]
    pub fn build() -> Self {
        let dfa = Self::build_c_lexer_dfa();
        Self { dfa }
    }

    /// Lexes the given source code and returns tokens or an error
    ///
    /// # Errors
    ///
    /// Returns [`LexError::UnexpectedCharacter`] when encountering a character that cannot
    /// be tokenized by the lexer's DFA.
    ///
    /// Returns [`LexError::InvalidNumber`] when a numeric literal cannot be parsed as a
    /// valid integer value (e.g., integer overflow).
    pub fn lex(&self, source: &str) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        let mut chars = source.char_indices().peekable();
        let mut line = 1;
        let mut column = 1;

        while chars.peek().is_some() {
            // Skip whitespace and update position
            let (_, ch) = chars.peek().unwrap_or_else(|| unreachable!());
            if ch.is_whitespace() {
                let (_, ch) = chars.next().unwrap_or_else(|| unreachable!());
                if ch == '\n' {
                    line += 1;
                    column = 1;
                } else {
                    column += 1;
                }
                continue;
            }

            // Try to match a token using the DFA
            let token_start_line = line;
            let token_start_column = column;
            let Some((token_byte_start, _)) = chars.peek().copied() else {
                break;
            };

            let mut current_state = self.dfa.start_state;
            let mut last_accepting_state = None;
            let mut last_accepting_pos = token_byte_start;
            let mut current_pos = token_byte_start;

            // Follow DFA transitions to find longest match
            let source_bytes = source.as_bytes();
            while current_pos < source_bytes.len() {
                let byte = source_bytes[current_pos];

                if let Some(next_state) = self.dfa.transition(current_state, byte) {
                    current_state = next_state;
                    current_pos += 1;

                    // Check if this state accepts a token
                    if let Some(state) = self.dfa.states.get(current_state) {
                        if state.token_type.is_some() {
                            last_accepting_state = Some(current_state);
                            last_accepting_pos = current_pos;
                        }
                    }
                } else {
                    break;
                }
            }

            if let Some(accepting_state) = last_accepting_state {
                if let Some(token_type) = &self.dfa.states[accepting_state].token_type {
                    let lexeme = String::from_utf8_lossy(
                        &source_bytes[token_byte_start..last_accepting_pos],
                    )
                    .to_string();

                    let final_token_type = match token_type {
                        TokenType::Identifier(_) => TokenType::Identifier(lexeme.clone()),
                        TokenType::LiteralInt(_) => match lexeme.parse::<i64>() {
                            Ok(value) => TokenType::LiteralInt(value),
                            Err(_) => {
                                return Err(LexError::InvalidNumber {
                                    text: lexeme.clone(),
                                    line: token_start_line,
                                    column: token_start_column,
                                });
                            }
                        },
                        _ => token_type.clone(),
                    };

                    tokens.push(Token {
                        token_type: final_token_type,
                        lexeme,
                        line: token_start_line,
                        column: token_start_column,
                    });

                    // Advance the character iterator to the end of the token
                    let token_text = &source[token_byte_start..last_accepting_pos];
                    for ch in token_text.chars() {
                        chars.next();
                        if ch == '\n' {
                            line += 1;
                            column = 1;
                        } else {
                            column += 1;
                        }
                    }
                } else {
                    unreachable!("Accepting state without token type");
                }
            } else {
                // No token matched
                let (_, ch) = chars.next().unwrap_or_else(|| unreachable!());
                return Err(LexError::UnexpectedCharacter {
                    character: ch,
                    line: token_start_line,
                    column: token_start_column,
                });
            }
        }

        Ok(tokens)
    }

    /// Builds the DFA for lexing C language tokens
    fn build_c_lexer_dfa() -> Dfa {
        use super::state_machine::Nfa;

        let literal = |literal: &[u8], token| {
            RegexPattern::new([RegexComponent::Literal(literal.to_vec())]).to_nfa(token)
        };

        let alpha_underscore = RegexComponent::Alternation(
            vec![RegexComponent::char_range(b'a', b'z')],
            vec![RegexComponent::Alternation(
                vec![RegexComponent::char_range(b'A', b'Z')],
                vec![RegexComponent::chars(b"_")],
            )],
        );
        let alphanum_underscore = RegexComponent::Alternation(
            vec![alpha_underscore.clone()],
            vec![RegexComponent::char_range(b'0', b'9')],
        );

        let nfas = vec![
            // Keywords
            literal(b"if", TokenType::If),
            literal(b"else", TokenType::Else),
            literal(b"for", TokenType::For),
            literal(b"while", TokenType::While),
            literal(b"do", TokenType::Do),
            literal(b"return", TokenType::Return),
            literal(b"switch", TokenType::Switch),
            literal(b"case", TokenType::Case),
            literal(b"break", TokenType::Break),
            literal(b"goto", TokenType::Goto),
            literal(b"void", TokenType::Void),
            literal(b"unsigned", TokenType::Unsigned),
            literal(b"char", TokenType::Char),
            literal(b"short", TokenType::Short),
            literal(b"int", TokenType::Int),
            literal(b"long", TokenType::Long),
            literal(b"float", TokenType::Float),
            literal(b"double", TokenType::Double),
            // Identifiers: [a-zA-Z_][a-zA-Z0-9_]*
            RegexPattern::new(&[
                alpha_underscore,
                RegexComponent::Repeat {
                    item: Box::new(alphanum_underscore),
                    min: 0,
                    max: None,
                },
            ])
            .to_nfa(TokenType::Identifier(String::new())),
            // Integer literals: [0-9]+
            RegexPattern::new(&[RegexComponent::Repeat {
                item: Box::new(RegexComponent::char_range(b'0', b'9')),
                min: 1,
                max: None,
            }])
            .to_nfa(TokenType::LiteralInt(0)),
            // Multi-character operators (must come before single-character ones)
            literal(b"++", TokenType::Incr),
            literal(b"--", TokenType::Decr),
            literal(b"<=", TokenType::CmpLeq),
            literal(b">=", TokenType::CmpGeq),
            literal(b"==", TokenType::CmpEq),
            literal(b"!=", TokenType::CmpNeq),
            literal(b"<<", TokenType::LShift),
            literal(b">>", TokenType::RShift),
            literal(b"*=", TokenType::StarEq),
            literal(b"/=", TokenType::SlashEq),
            literal(b"+=", TokenType::PlusEq),
            literal(b"-=", TokenType::MinusEq),
            literal(b"<<=", TokenType::LShiftEq),
            literal(b">>=", TokenType::RShiftEq),
            // Single-character operators and punctuation
            literal(b"<", TokenType::CmpLt),
            literal(b">", TokenType::CmpGt),
            literal(b"*", TokenType::Star),
            literal(b"/", TokenType::Slash),
            literal(b"+", TokenType::Plus),
            literal(b"-", TokenType::Minus),
            literal(b"=", TokenType::Eq),
            literal(b"&", TokenType::Ampersand),
            literal(b";", TokenType::Semicolon),
            literal(b":", TokenType::Colon),
            literal(b",", TokenType::Comma),
            literal(b"(", TokenType::LParen),
            literal(b")", TokenType::RParen),
            literal(b"[", TokenType::LSquare),
            literal(b"]", TokenType::RSquare),
            literal(b"{", TokenType::LBrace),
            literal(b"}", TokenType::RBrace),
        ];

        // Merge all NFAs into one and convert to minimized DFA
        Nfa::merge(&nfas).to_dfa().minimize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    /// Helper function to create a token for testing
    fn token(token_type: TokenType, lexeme: &str, line: usize, column: usize) -> Token {
        Token {
            token_type,
            lexeme: lexeme.to_string(),
            line,
            column,
        }
    }

    /// Helper function to lex source and expect success
    fn lex_success(source: &str) -> Vec<Token> {
        let lexer = Lexer::build();
        lexer.lex(source).expect("Lexing should succeed")
    }

    /// Helper function to lex source and expect error
    fn lex_error(source: &str) -> LexError {
        let lexer = Lexer::build();
        lexer.lex(source).expect_err("Lexing should fail")
    }

    /// Helper function to lex a series of single tokens, expecting each to be successful
    fn lex_each(inputs: &[(&str, TokenType)]) {
        for (input, token_type) in inputs {
            assert_eq!(
                lex_success(input),
                vec![token(token_type.clone(), input, 1, 1)]
            );
        }
    }

    #[test]
    fn test_empty_input() {
        let tokens = lex_success("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_whitespace_only() {
        let tokens = lex_success("   \t\n  \r\n  ");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_single_identifier() {
        lex_each(&[("hello", TokenType::Identifier("hello".to_string()))]);
    }

    #[test]
    fn test_single_integer() {
        lex_each(&[("42", TokenType::LiteralInt(42))]);
    }

    #[test]
    fn test_single_operator() {
        lex_each(&[("+", TokenType::Plus)]);
    }

    // Keyword recognition tests
    #[test]
    fn test_control_flow_keywords() {
        lex_each(&[
            ("if", TokenType::If),
            ("else", TokenType::Else),
            ("for", TokenType::For),
            ("while", TokenType::While),
            ("do", TokenType::Do),
            ("return", TokenType::Return),
            ("switch", TokenType::Switch),
            ("case", TokenType::Case),
            ("break", TokenType::Break),
            ("goto", TokenType::Goto),
        ]);
    }

    #[test]
    fn test_type_keywords() {
        lex_each(&[
            ("void", TokenType::Void),
            ("unsigned", TokenType::Unsigned),
            ("char", TokenType::Char),
            ("short", TokenType::Short),
            ("int", TokenType::Int),
            ("long", TokenType::Long),
            ("float", TokenType::Float),
            ("double", TokenType::Double),
        ]);
    }

    #[test]
    fn test_keyword_vs_identifier() {
        lex_each(&[
            ("if", TokenType::If),
            ("iff", TokenType::Identifier("iff".to_string())),
            ("if_var", TokenType::Identifier("if_var".to_string())),
            ("var_if", TokenType::Identifier("var_if".to_string())),
            ("If", TokenType::Identifier("If".to_string())),
            ("Int", TokenType::Identifier("Int".to_string())),
        ]);
    }

    // Operator and punctuation tests
    #[test]
    fn test_single_character_operators() {
        lex_each(&[
            ("<", TokenType::CmpLt),
            (">", TokenType::CmpGt),
            ("*", TokenType::Star),
            ("/", TokenType::Slash),
            ("+", TokenType::Plus),
            ("-", TokenType::Minus),
            ("=", TokenType::Eq),
            ("&", TokenType::Ampersand),
        ]);
    }

    #[test]
    fn test_multi_character_operators() {
        lex_each(&[
            ("++", TokenType::Incr),
            ("--", TokenType::Decr),
            ("<=", TokenType::CmpLeq),
            (">=", TokenType::CmpGeq),
            ("==", TokenType::CmpEq),
            ("!=", TokenType::CmpNeq),
            ("<<", TokenType::LShift),
            (">>", TokenType::RShift),
            ("*=", TokenType::StarEq),
            ("/=", TokenType::SlashEq),
            ("+=", TokenType::PlusEq),
            ("-=", TokenType::MinusEq),
            ("<<=", TokenType::LShiftEq),
            (">>=", TokenType::RShiftEq),
        ]);
    }

    #[test]
    fn test_punctuation() {
        lex_each(&[
            (";", TokenType::Semicolon),
            (":", TokenType::Colon),
            (",", TokenType::Comma),
            ("(", TokenType::LParen),
            (")", TokenType::RParen),
            ("[", TokenType::LSquare),
            ("]", TokenType::RSquare),
            ("{", TokenType::LBrace),
            ("}", TokenType::RBrace),
        ]);
    }

    #[test]
    fn test_operator_precedence_longest_match() {
        // Test that longer operators take precedence over shorter ones
        lex_each(&[
            ("++", TokenType::Incr),
            ("<=", TokenType::CmpLeq),
            ("<<=", TokenType::LShiftEq),
        ]);

        // Test that separate operators are parsed separately
        let tokens = lex_success("< =");
        assert_eq!(
            tokens,
            vec![
                token(TokenType::CmpLt, "<", 1, 1),
                token(TokenType::Eq, "=", 1, 3),
            ]
        );
    }

    // Identifier and literal tests
    #[test]
    fn test_valid_identifiers() {
        lex_each(&[
            ("x", TokenType::Identifier("x".to_string())),
            ("variable", TokenType::Identifier("variable".to_string())),
            ("var_name", TokenType::Identifier("var_name".to_string())),
            ("_private", TokenType::Identifier("_private".to_string())),
            ("camelCase", TokenType::Identifier("camelCase".to_string())),
            (
                "PascalCase",
                TokenType::Identifier("PascalCase".to_string()),
            ),
            ("var123", TokenType::Identifier("var123".to_string())),
            ("_", TokenType::Identifier("_".to_string())),
            ("a1b2c3", TokenType::Identifier("a1b2c3".to_string())),
            ("CONSTANT", TokenType::Identifier("CONSTANT".to_string())),
        ]);
    }

    #[test]
    fn test_integer_literals() {
        lex_each(&[
            ("0", TokenType::LiteralInt(0)),
            ("1", TokenType::LiteralInt(1)),
            ("42", TokenType::LiteralInt(42)),
            ("123456", TokenType::LiteralInt(123_456)),
            ("9999999999", TokenType::LiteralInt(9_999_999_999)),
        ]);
    }

    #[test]
    fn test_identifier_vs_number_distinction() {
        // Numbers starting with digits
        let tokens = lex_success("123abc");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token_type, TokenType::LiteralInt(123));
        assert_eq!(
            tokens[1].token_type,
            TokenType::Identifier("abc".to_string())
        );

        // Identifiers starting with letters
        let tokens = lex_success("abc123");
        assert_eq!(tokens.len(), 1);
        assert_eq!(
            tokens[0].token_type,
            TokenType::Identifier("abc123".to_string())
        );

        // Identifiers starting with underscore
        let tokens = lex_success("_123");
        assert_eq!(tokens.len(), 1);
        assert_eq!(
            tokens[0].token_type,
            TokenType::Identifier("_123".to_string())
        );
    }

    #[test]
    fn test_zero_and_leading_zeros() {
        // Single zero
        lex_each(&[("0", TokenType::LiteralInt(0))]);

        // Numbers with leading zeros - the lexer actually parses "00123" as a single number
        lex_each(&[("00123", TokenType::LiteralInt(123))]);

        // Test separate zeros
        let tokens = lex_success("0 0 123");
        assert_eq!(
            tokens,
            vec![
                token(TokenType::LiteralInt(0), "0", 1, 1),
                token(TokenType::LiteralInt(0), "0", 1, 3),
                token(TokenType::LiteralInt(123), "123", 1, 5),
            ]
        );
    }

    // Error handling tests
    #[test]
    fn test_unexpected_character_errors() {
        let test_cases = [
            ("@", '@'),
            ("#", '#'),
            ("$", '$'),
            ("%", '%'),
            ("^", '^'),
            ("~", '~'),
            ("`", '`'),
            ("\\", '\\'),
            ("?", '?'),
            (".", '.'),
        ];

        for (input, expected_char) in test_cases {
            let error = lex_error(input);
            match error {
                LexError::UnexpectedCharacter {
                    character,
                    line,
                    column,
                } => {
                    assert_eq!(character, expected_char);
                    assert_eq!(line, 1);
                    assert_eq!(column, 1);
                }
                _ => panic!("Expected `UnexpectedCharacter` error for '{input}'"),
            }
        }
    }

    #[test]
    fn test_integer_overflow_error() {
        let large_number = "99999999999999999999999999999999";
        let error = lex_error(large_number);
        match error {
            LexError::InvalidNumber { text, line, column } => {
                assert_eq!(text, large_number);
                assert_eq!(line, 1);
                assert_eq!(column, 1);
            }
            _ => panic!("Expected `InvalidNumber` error for large number"),
        }
    }

    #[test]
    fn test_error_position_tracking() {
        // Test error position tracking across multiple lines
        let source = "int x = 42;\n@";
        let error = lex_error(source);
        match error {
            LexError::UnexpectedCharacter {
                character,
                line,
                column,
            } => {
                assert_eq!(character, '@');
                assert_eq!(line, 2);
                assert_eq!(column, 1);
            }
            _ => panic!("Expected `UnexpectedCharacter` error"),
        }

        // Test error in middle of line
        let source = "int @ = 42;";
        let error = lex_error(source);
        match error {
            LexError::UnexpectedCharacter {
                character,
                line,
                column,
            } => {
                assert_eq!(character, '@');
                assert_eq!(line, 1);
                assert_eq!(column, 5);
            }
            _ => panic!("Expected `UnexpectedCharacter` error"),
        }
    }

    #[test]
    fn test_partial_success_before_error() {
        // Lex should return tokens up until the error
        let lexer = Lexer::build();
        let result = lexer.lex("int x @");

        match result {
            Err(LexError::UnexpectedCharacter {
                character,
                line,
                column,
            }) => {
                assert_eq!(character, '@');
                assert_eq!(line, 1);
                assert_eq!(column, 7);
            }
            _ => panic!("Expected `UnexpectedCharacter` error"),
        }
    }

    // Position tracking tests
    #[test]
    fn test_single_line_positions() {
        let tokens = lex_success("int x = 42;");

        assert_eq!(
            tokens,
            vec![
                token(TokenType::Int, "int", 1, 1),
                token(TokenType::Identifier("x".to_string()), "x", 1, 5),
                token(TokenType::Eq, "=", 1, 7),
                token(TokenType::LiteralInt(42), "42", 1, 9),
                token(TokenType::Semicolon, ";", 1, 11),
            ]
        );
    }

    #[test]
    fn test_multi_line_positions() {
        let source = "int x;\nchar y;\nreturn 0;";
        let tokens = lex_success(source);

        assert_eq!(
            tokens,
            vec![
                token(TokenType::Int, "int", 1, 1),
                token(TokenType::Identifier("x".to_string()), "x", 1, 5),
                token(TokenType::Semicolon, ";", 1, 6),
                token(TokenType::Char, "char", 2, 1),
                token(TokenType::Identifier("y".to_string()), "y", 2, 6),
                token(TokenType::Semicolon, ";", 2, 7),
                token(TokenType::Return, "return", 3, 1),
                token(TokenType::LiteralInt(0), "0", 3, 8),
                token(TokenType::Semicolon, ";", 3, 9)
            ]
        );
    }

    #[test]
    fn test_whitespace_position_tracking() {
        let source = "  int   x  =\t42  ;  ";
        let tokens = lex_success(source);

        assert_eq!(
            tokens,
            vec![
                token(TokenType::Int, "int", 1, 3),
                token(TokenType::Identifier("x".to_string()), "x", 1, 9),
                token(TokenType::Eq, "=", 1, 12),
                token(TokenType::LiteralInt(42), "42", 1, 14),
                token(TokenType::Semicolon, ";", 1, 18),
            ]
        );
    }

    #[test]
    fn test_mixed_newline_types() {
        // Test different newline types: \n, \r\n
        let source = "int x;\nchar y;\r\nfloat z;";
        let tokens = lex_success(source);

        assert_eq!(
            tokens,
            vec![
                token(TokenType::Int, "int", 1, 1),
                token(TokenType::Identifier("x".to_string()), "x", 1, 5),
                token(TokenType::Semicolon, ";", 1, 6),
                token(TokenType::Char, "char", 2, 1),
                token(TokenType::Identifier("y".to_string()), "y", 2, 6),
                token(TokenType::Semicolon, ";", 2, 7),
                token(TokenType::Float, "float", 3, 1),
                token(TokenType::Identifier("z".to_string()), "z", 3, 7),
                token(TokenType::Semicolon, ";", 3, 8),
            ]
        );
    }

    #[test]
    fn test_empty_lines() {
        let source = "int x;\n\n\nchar y;";
        let tokens = lex_success(source);

        assert_eq!(
            tokens,
            vec![
                token(TokenType::Int, "int", 1, 1),
                token(TokenType::Identifier("x".to_string()), "x", 1, 5),
                token(TokenType::Semicolon, ";", 1, 6),
                token(TokenType::Char, "char", 4, 1),
                token(TokenType::Identifier("y".to_string()), "y", 4, 6),
                token(TokenType::Semicolon, ";", 4, 7),
            ]
        );
    }

    // Complex integration tests
    #[test]
    fn test_simple_c_program() {
        let source = "int main() { return 42; }";
        let tokens = lex_success(source);

        let expected = vec![
            token(TokenType::Int, "int", 1, 1),
            token(TokenType::Identifier("main".to_string()), "main", 1, 5),
            token(TokenType::LParen, "(", 1, 9),
            token(TokenType::RParen, ")", 1, 10),
            token(TokenType::LBrace, "{", 1, 12),
            token(TokenType::Return, "return", 1, 14),
            token(TokenType::LiteralInt(42), "42", 1, 21),
            token(TokenType::Semicolon, ";", 1, 23),
            token(TokenType::RBrace, "}", 1, 25),
        ];

        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_function_with_parameters() {
        let source = "int add(int a, int b) {\n    return a + b;\n}";
        let tokens = lex_success(source);

        assert_eq!(
            tokens,
            vec![
                token(TokenType::Int, "int", 1, 1),
                token(TokenType::Identifier("add".to_string()), "add", 1, 5),
                token(TokenType::LParen, "(", 1, 8),
                token(TokenType::Int, "int", 1, 9),
                token(TokenType::Identifier("a".to_string()), "a", 1, 13),
                token(TokenType::Comma, ",", 1, 14),
                token(TokenType::Int, "int", 1, 16),
                token(TokenType::Identifier("b".to_string()), "b", 1, 20),
                token(TokenType::RParen, ")", 1, 21),
                token(TokenType::LBrace, "{", 1, 23),
                token(TokenType::Return, "return", 2, 5),
                token(TokenType::Identifier("a".to_string()), "a", 2, 12),
                token(TokenType::Plus, "+", 2, 14),
                token(TokenType::Identifier("b".to_string()), "b", 2, 16),
                token(TokenType::Semicolon, ";", 2, 17),
                token(TokenType::RBrace, "}", 3, 1),
            ]
        );
    }

    #[test]
    fn test_complex_expression() {
        let source = "x = (a + b) * c - d++;";
        let tokens = lex_success(source);

        let expected = vec![
            token(TokenType::Identifier("x".to_string()), "x", 1, 1),
            token(TokenType::Eq, "=", 1, 3),
            token(TokenType::LParen, "(", 1, 5),
            token(TokenType::Identifier("a".to_string()), "a", 1, 6),
            token(TokenType::Plus, "+", 1, 8),
            token(TokenType::Identifier("b".to_string()), "b", 1, 10),
            token(TokenType::RParen, ")", 1, 11),
            token(TokenType::Star, "*", 1, 13),
            token(TokenType::Identifier("c".to_string()), "c", 1, 15),
            token(TokenType::Minus, "-", 1, 17),
            token(TokenType::Identifier("d".to_string()), "d", 1, 19),
            token(TokenType::Incr, "++", 1, 20),
            token(TokenType::Semicolon, ";", 1, 22),
        ];

        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_control_flow_statements() {
        let source = "if (x <= 10) {\n    for (int i = 0; i < x; i++) {\n        break;\n    }\n}";
        let tokens = lex_success(source);

        assert_eq!(
            tokens,
            vec![
                token(TokenType::If, "if", 1, 1),
                token(TokenType::LParen, "(", 1, 4),
                token(TokenType::Identifier("x".to_string()), "x", 1, 5),
                token(TokenType::CmpLeq, "<=", 1, 7),
                token(TokenType::LiteralInt(10), "10", 1, 10),
                token(TokenType::RParen, ")", 1, 12),
                token(TokenType::LBrace, "{", 1, 14),
                token(TokenType::For, "for", 2, 5),
                token(TokenType::LParen, "(", 2, 9),
                token(TokenType::Int, "int", 2, 10),
                token(TokenType::Identifier("i".to_string()), "i", 2, 14),
                token(TokenType::Eq, "=", 2, 16),
                token(TokenType::LiteralInt(0), "0", 2, 18),
                token(TokenType::Semicolon, ";", 2, 19),
                token(TokenType::Identifier("i".to_string()), "i", 2, 21),
                token(TokenType::CmpLt, "<", 2, 23),
                token(TokenType::Identifier("x".to_string()), "x", 2, 25),
                token(TokenType::Semicolon, ";", 2, 26),
                token(TokenType::Identifier("i".to_string()), "i", 2, 28),
                token(TokenType::Incr, "++", 2, 29),
                token(TokenType::RParen, ")", 2, 31),
                token(TokenType::LBrace, "{", 2, 33),
                token(TokenType::Break, "break", 3, 9),
                token(TokenType::Semicolon, ";", 3, 14),
                token(TokenType::RBrace, "}", 4, 5),
                token(TokenType::RBrace, "}", 5, 1),
            ]
        );
    }

    #[test]
    fn test_adjacent_operators() {
        let source = "a++=b--*c<<=d>>=e";
        let tokens = lex_success(source);

        let expected = vec![
            token(TokenType::Identifier("a".to_string()), "a", 1, 1),
            token(TokenType::Incr, "++", 1, 2),
            token(TokenType::Eq, "=", 1, 4),
            token(TokenType::Identifier("b".to_string()), "b", 1, 5),
            token(TokenType::Decr, "--", 1, 6),
            token(TokenType::Star, "*", 1, 8),
            token(TokenType::Identifier("c".to_string()), "c", 1, 9),
            token(TokenType::LShiftEq, "<<=", 1, 10),
            token(TokenType::Identifier("d".to_string()), "d", 1, 13),
            token(TokenType::RShiftEq, ">>=", 1, 14),
            token(TokenType::Identifier("e".to_string()), "e", 1, 17),
        ];

        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_large_src() {
        fn check_line(tokens: &[Token], line_no: usize) {
            let var_name = format!("var_{line_no}");
            let var_val = (line_no * 2).to_string();
            assert_eq!(
                &tokens[(5 * line_no)..(5 * (line_no + 1))],
                &[
                    token(TokenType::Int, "int", line_no + 1, 1),
                    token(
                        TokenType::Identifier(var_name.clone()),
                        &var_name,
                        line_no + 1,
                        5
                    ),
                    token(TokenType::Eq, "=", line_no + 1, var_name.len() + 6),
                    token(
                        #[allow(clippy::cast_possible_wrap)]
                        TokenType::LiteralInt((line_no * 2) as i64),
                        &var_val,
                        line_no + 1,
                        var_name.len() + 8
                    ),
                    token(
                        TokenType::Semicolon,
                        ";",
                        line_no + 1,
                        var_name.len() + var_val.len() + 8
                    ),
                ]
            );
        }

        let mut source = String::new();
        for i in 0..100 {
            writeln!(source, "int var_{} = {};", i, i * 2).unwrap_or_else(|_| unreachable!());
        }

        let tokens = lex_success(&source);
        assert_eq!(tokens.len(), 500); // 5 tokens per line * 100 lines

        check_line(&tokens, 0);
        check_line(&tokens, 1);
        check_line(&tokens, 2);
        check_line(&tokens, 47);
        check_line(&tokens, 97);
        check_line(&tokens, 98);
        check_line(&tokens, 99);
    }
}
