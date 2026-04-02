use std::collections::HashMap;
use std::{collections::VecDeque, io::BufRead};

use crate::pp_lexer::{PpLexer, PpLexerError};
use crate::span::Spanned;
use crate::token::{Identifier, PreprocessingToken};

#[derive(Debug)]
pub enum PreprocessorError<'src> {
    PpLexerError(PpLexerError<'src>),
}

impl<'src> From<PpLexerError<'src>> for PreprocessorError<'src> {
    fn from(e: PpLexerError<'src>) -> Self {
        Self::PpLexerError(e)
    }
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
    macros: HashMap<String, Vec<PreprocessingToken>>,
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

    fn try_read_directive(&mut self) -> Result<Option<Directive<'_>>, PreprocessorError<'src>> {
        let mut buffer = Vec::new();

        macro_rules! read_token {
            () => {{
                match self.pp_lexer.next() {
                    Some(Ok(t)) => {
                        buffer.push(t.clone());
                        t
                    }
                    Some(Err(e)) => {
                        self.output.extend(buffer);
                        return Err(e.into());
                    }
                    None => {
                        self.output.extend(buffer);
                        return Ok(None);
                    }
                }
            }};
        }

        // Step 1: Read newline
        let nl = read_token!();

        if !matches!(nl.value, PreprocessingToken::Newline) {
            self.output.extend(buffer);
            return Ok(None);
        }

        // Step 2: Read next token (optional whitespace or hash)
        let ws_hash = read_token!();

        // Check if we have whitespace before hash
        let hash = if matches!(ws_hash.value, PreprocessingToken::Whitespace) {
            read_token!()
        } else {
            ws_hash
        };

        // Step 3: Check for hash
        if !matches!(
            hash.value,
            PreprocessingToken::Punctuator(crate::token::Punctuator::Hash)
        ) {
            buffer.push(hash);
            self.output.extend(buffer);
            return Ok(None);
        }
        buffer.push(hash);

        // Step 4: Read identifier
        let identifier = read_token!();

        let identifier = if let PreprocessingToken::Identifier(ref id) = identifier.value {
            id.clone()
        } else {
            buffer.push(identifier);
            self.output.extend(buffer);
            return Ok(None);
        };

        // Step 5: Read rest of tokens on this line
        let mut rest_tokens = Vec::new();
        loop {
            match self.pp_lexer.next() {
                None
                | Some(Ok(Spanned {
                    span: _,
                    value: PreprocessingToken::Newline,
                })) => {
                    break;
                }
                Some(Ok(token)) => {
                    rest_tokens.push(token);
                }
                Some(Err(e)) => {
                    self.output.extend(buffer);
                    return Err(e.into());
                }
            }
        }

        Ok(Some((identifier, rest_tokens)))
    }

    fn process_next(
        &mut self,
    ) -> Option<Result<Spanned<'src, PreprocessingToken>, PreprocessorError<'src>>> {
        match self.try_read_directive() {
            Ok(Some((_identifier, _rest_tokens))) => {
                // TODO: Actually process directives
                self.process_next()
            }
            Ok(None) => self.output.pop_front().map(Ok),
            Err(e) => Some(Err(e)),
        }
    }
}

impl<'src, R: BufRead> Iterator for Preprocessor<'src, R> {
    type Item = Result<Spanned<'src, PreprocessingToken>, PreprocessorError<'src>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.output
            .pop_front()
            .map_or_else(|| self.process_next(), |t| Some(Ok(t)))
    }
}
