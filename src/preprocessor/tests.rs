use super::*;
use crate::source_reader::SourceReader;
use std::io::Cursor;

fn preprocess(input: &str) -> Vec<PreprocessingToken> {
    let reader = SourceReader::new("<test>", Cursor::new(input.as_bytes().to_vec()));
    let pp_lexer = PpLexer::new(reader);

    Preprocessor::new(pp_lexer, "<test>")
        .map(|token| token.unwrap().value)
        .collect()
}

fn identifiers(tokens: &[PreprocessingToken]) -> Vec<&str> {
    tokens
        .iter()
        .filter_map(|token| match token {
            PreprocessingToken::Identifier(identifier) => Some(identifier.as_ref()),
            _ => None,
        })
        .collect()
}

#[test]
fn outputs_only_active_ifdef_branch() {
    let tokens = preprocess(
        "#define FOO\n\
             #ifdef FOO\n\
             yes\n\
             #else\n\
             no\n\
             #endif\n\
             #ifndef FOO\n\
             also_no\n\
             #else\n\
             also_yes\n\
             #endif\n",
    );

    assert_eq!(identifiers(&tokens), vec!["yes", "also_yes"]);
}

#[test]
fn inactive_branches_do_not_define_macros() {
    let tokens = preprocess(
        "#ifdef MISSING\n\
             #define SELECTED wrong\n\
             #else\n\
             SELECTED\n\
             #endif\n",
    );

    assert_eq!(identifiers(&tokens), vec!["SELECTED"]);
}

#[test]
fn undef_removes_macro() {
    let tokens = preprocess(
        "#define FOO bar\n\
             #undef FOO\n\
             FOO\n",
    );

    assert_eq!(identifiers(&tokens), vec!["FOO"]);
}

#[test]
fn inactive_branches_do_not_undef_macros() {
    let tokens = preprocess(
        "#define FOO bar\n\
             #ifdef MISSING\n\
             #undef FOO\n\
             #endif\n\
             FOO\n",
    );

    assert_eq!(identifiers(&tokens), vec!["bar"]);
}

#[test]
fn nested_conditionals_obey_inactive_parents() {
    let tokens = preprocess(
        "#ifdef OUTER\n\
             #ifdef INNER\n\
             hidden_inner\n\
             #else\n\
             hidden_else\n\
             #endif\n\
             #else\n\
             visible\n\
             #endif\n",
    );

    assert_eq!(identifiers(&tokens), vec!["visible"]);
}
