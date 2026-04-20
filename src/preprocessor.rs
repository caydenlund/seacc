use std::collections::HashMap;
use std::{collections::VecDeque, io::BufRead};

use crate::pp_lexer::{PpLexer, PpLexerError};
use crate::span::Spanned;
use crate::token::{Identifier, PreprocessingToken, Punctuator};

#[derive(Debug)]
pub enum PreprocessorError<'src> {
    PpLexerError(PpLexerError<'src>),
}

impl<'src> From<PpLexerError<'src>> for PreprocessorError<'src> {
    fn from(e: PpLexerError<'src>) -> Self {
        Self::PpLexerError(e)
    }
}

#[derive(Clone, Debug)]
enum Macro {
    ObjectLike(Vec<PreprocessingToken>),
    FunctionLike(),
}

/// The main preprocessor (translation phase 4)
///
/// Processes preprocessing directives, expands macros, handles includes,
/// and performs conditional compilation.
pub struct Preprocessor<'src, R: BufRead> {
    pp_lexer: PpLexer<'src, R>,
    curr_file: &'src str,
    include_stack: Vec<String>,
    output: VecDeque<Spanned<'src, PreprocessingToken>>,
    macros: HashMap<String, Macro>,
}

type Directive<'src> = (Identifier, Vec<Spanned<'src, PreprocessingToken>>);

impl<'src, R: BufRead> Preprocessor<'src, R> {
    pub fn new(pp_lexer: PpLexer<'src, R>, curr_file: &'src str) -> Self {
        Self {
            pp_lexer,
            curr_file,
            include_stack: Vec::new(),
            output: VecDeque::new(),
            macros: HashMap::new(),
        }
    }

    fn process_tokens(
        &mut self,
        to_process: impl IntoIterator<Item = Spanned<'src, PreprocessingToken>>,
    ) {
        for token in to_process {
            if let PreprocessingToken::Identifier(ref ident) = token.value
                && let Some(r#macro) = self.macros.get(&ident.to_string())
            {
                match r#macro {
                    Macro::ObjectLike(replacements) => {
                        self.output
                            .extend(replacements.iter().cloned().map(|value| Spanned {
                                value,
                                span: token.span,
                            }));
                    }

                    Macro::FunctionLike() => todo!("unhandled function-like macro"),
                }
            } else {
                self.output.push_back(token);
            }
        }
    }

    fn match_directive(
        to_process: &VecDeque<Spanned<'src, PreprocessingToken>>,
    ) -> Option<Directive<'src>> {
        let mut ind = 0;

        // 1: ( )?  #
        match (
            to_process.get(ind).map(|t| &t.value),
            to_process.get(ind + 1).map(|t| &t.value),
        ) {
            (
                Some(PreprocessingToken::Whitespace),
                Some(PreprocessingToken::Punctuator(Punctuator::Hash)),
            ) => ind += 2,
            (Some(PreprocessingToken::Punctuator(Punctuator::Hash)), Some(_)) => ind += 1,
            _ => return None,
        }

        // 2: ( )?  Identifier
        let ident = match (
            to_process.get(ind).map(|t| &t.value),
            to_process.get(ind + 1).map(|t| &t.value),
        ) {
            (Some(PreprocessingToken::Whitespace), Some(PreprocessingToken::Identifier(s))) => {
                ind += 2;
                s.clone()
            }
            (Some(PreprocessingToken::Identifier(s)), _) => {
                ind += 1;
                s.clone()
            }
            _ => return None,
        };

        // 3: other tokens
        let others = to_process.iter().skip(ind).cloned().collect();

        Some((ident, others))
    }

    fn process_line(&mut self) -> Option<Result<(), PreprocessorError<'src>>> {
        let mut to_process = VecDeque::new();
        for tok in self.pp_lexer.by_ref() {
            match tok {
                Ok(tok) => {
                    let is_newline = matches!(tok.value, PreprocessingToken::Newline);
                    to_process.push_back(tok);
                    if is_newline {
                        break;
                    }
                }
                Err(e) => return Some(Err(e.into())),
            }
        }
        if to_process.is_empty() {
            return None;
        }

        if let Some((ident, rest)) = Self::match_directive(&to_process) {
            match ident.to_string().as_ref() {
                "define" => {
                    let mut rest = rest
                        .into_iter()
                        .skip_while(|t| t.value == PreprocessingToken::Whitespace);

                    let Some(PreprocessingToken::Identifier(ident)) = rest.next().map(|t| t.value)
                    else {
                        panic!("invalid #define at {:?}", to_process.front().unwrap().span);
                    };

                    match rest.next().map(|t| t.value) {
                        Some(PreprocessingToken::Whitespace | PreprocessingToken::Newline)
                        | None => {
                            self.macros.insert(
                                ident.into_string(),
                                Macro::ObjectLike(
                                    rest.map(|t| t.value)
                                        .take_while(|t| *t != PreprocessingToken::Newline)
                                        .collect(),
                                ),
                            );
                        }
                        Some(PreprocessingToken::Punctuator(Punctuator::LParen)) => {
                            todo!(
                                "unhandled macro-like #define at {:?}",
                                to_process.front().unwrap().span
                            );
                        }
                        _ => unreachable!(
                            "invalid #define at {:?}",
                            to_process.front().unwrap().span
                        ),
                    }
                }
                "undef" => todo!(),

                _ => panic!("invalid directive {ident}"),
            }

            return Some(Ok(()));
        }

        self.process_tokens(to_process);

        Some(Ok(()))
    }
}

impl<'src, R: BufRead> Iterator for Preprocessor<'src, R> {
    type Item = Result<Spanned<'src, PreprocessingToken>, PreprocessorError<'src>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(tok) = self.output.pop_front() {
            return Some(Ok(tok));
        }
        match self.process_line() {
            Some(Err(e)) => Some(Err(e)),
            Some(Ok(())) => self.next(),
            None => None,
        }
    }
}
