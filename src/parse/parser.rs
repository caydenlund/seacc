use std::{iter::Peekable, str::Chars};

use crate::parse::{Expr, ExprArena, ExprId, ParseResult};

#[derive(Debug, Clone)]
pub(super) struct Parser<'src> {
    input: Peekable<Chars<'src>>,
    exprs: ExprArena,
}

impl<'src> Parser<'src> {
    pub(super) fn new(src: &'src str) -> Self {
        Self {
            input: src.chars().peekable(),
            exprs: ExprArena::new(),
        }
    }

    pub(super) fn parse(mut self) -> Result<ParseResult, String> {
        let root_expr = self.parse_expr()?;
        self.skip_ws();

        if let Some(ch) = self.input.peek() {
            return Err(format!("unexpected input after expression: '{ch}'"));
        }

        Ok(ParseResult {
            exprs: self.exprs,
            root_expr,
        })
    }

    fn parse_expr(&mut self) -> Result<ExprId, String> {
        self.skip_ws();
        match self.peek() {
            Some(&'(') => self.parse_list(),

            Some(&')') => Err("unexpected ')'".to_string()),
            Some(&'"') => self.parse_string(),
            Some(&'#') => self.parse_hash(),

            Some(ch) if ch.is_ascii_digit() => self.parse_int(),
            Some(ch) => Err(format!("unexpected char {ch}")),
            None => Err("unexpected end of input".to_string()),
        }
    }

    fn parse_list(&mut self) -> Result<ExprId, String> {
        assert_eq!(self.next(), Some('('));
        self.skip_ws();

        let mut exprs = Vec::new();
        loop {
            self.skip_ws();

            match self.peek() {
                Some(')') => {
                    self.next();
                    return Ok(self.exprs.insert(Expr::List(exprs)));
                }
                Some(_) => {
                    exprs.push(self.parse_expr()?);
                }
                None => return Err("unterminated list".into()),
            }
        }
    }

    fn parse_string(&mut self) -> Result<ExprId, String> {
        assert_eq!(self.next(), Some('"'));
        let mut sb = String::new();

        let mut escaping = false;
        loop {
            if escaping {
                match self.next() {
                    Some('n') => sb.push('\n'),
                    Some('t') => sb.push('\t'),
                    Some('\\') => sb.push('\\'),
                    Some(ch) => return Err(format!("invalid string escape sequence: '\\{ch}'")),
                    None => return Err("unterminated string".into()),
                }
                escaping = false;
            } else {
                match self.next() {
                    Some('\\') => escaping = true,
                    Some('"') => return Ok(self.exprs.insert(Expr::String(sb))),
                    Some(ch) => sb.push(ch),
                    None => return Err("unterminated string".into()),
                }
            }
        }
    }

    fn parse_hash(&mut self) -> Result<ExprId, String> {
        assert_eq!(self.next(), Some('#'));
        if self.next_if_eq('f') {
            Ok(self.exprs.insert(Expr::Bool(false)))
        } else if self.next_if_eq('t') {
            Ok(self.exprs.insert(Expr::Bool(true)))
        } else if self.next_if_eq('\\') {
            if self.next_if_eq('s') {
                if self.next_if_eq('p') {
                    if self.next_if_eq('a') && self.next_if_eq('c') && self.next_if_eq('e') {
                        Ok(self.exprs.insert(Expr::Char(' ')))
                    } else {
                        Err("invalid '#space' sequence".into())
                    }
                } else if self.next_if(|ch| ch.is_whitespace()).is_some() {
                    Ok(self.exprs.insert(Expr::Char('s')))
                } else {
                    Err("invalid '#space' sequence".into())
                }
            } else if let Some(ch) = self.next() {
                Ok(self.exprs.insert(Expr::Char(ch)))
            } else {
                Err("unterminated char hash".into())
            }
        } else if let Some(ch) = self.next() {
            Err(format!("invalid char after hash: '{ch}'"))
        } else {
            Err("unterminated hash".into())
        }
    }

    fn parse_int(&mut self) -> Result<ExprId, String> {
        let mut s = String::new();
        while let Some(ch) = self.next_if(char::is_ascii_digit) {
            s.push(ch);
        }
        let Ok(n) = s.parse::<i64>() else {
            return Err(format!("unable to parse integer from {s}"));
        };
        Ok(self.exprs.insert(Expr::Integer(n)))
    }

    fn skip_ws(&mut self) {
        while self.next_if(|ch| ch.is_whitespace()).is_some() {}
    }

    fn next_if_eq(&mut self, ch: char) -> bool {
        if self.peek() == Some(&ch) {
            self.next();
            true
        } else {
            false
        }
    }

    fn next_if(&mut self, f: impl Fn(&char) -> bool) -> Option<char> {
        if self.peek().is_some_and(f) {
            self.next()
        } else {
            None
        }
    }

    fn next(&mut self) -> Option<char> {
        self.input.next()
    }

    fn peek(&mut self) -> Option<&char> {
        self.input.peek()
    }
}
