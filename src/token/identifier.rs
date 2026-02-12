use std::{borrow::Borrow, error::Error, fmt::Display};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identifier(String);

impl Identifier {
    pub fn new(raw_ident: &str) -> Result<Self, IdentifierError> {
        let err = || Err(IdentifierError(raw_ident.into()));

        let mut chars = raw_ident.chars();
        let Some(first) = chars.next() else {
            return err();
        };

        if !first.is_alphabetic() && first != '_' {
            return Err(IdentifierError(raw_ident.into()));
        }

        for ch in chars {
            if !ch.is_alphanumeric() && ch != '_' {
                return Err(IdentifierError(raw_ident.into()));
            }
        }

        Ok(Self(raw_ident.into()))
    }

    pub unsafe fn new_unchecked(raw_ident: &str) -> Self {
        Self(raw_ident.into())
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl TryFrom<&str> for Identifier {
    type Error = IdentifierError;

    fn try_from(raw_ident: &str) -> Result<Self, Self::Error> {
        Self::new(raw_ident)
    }
}

impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for Identifier {
    fn borrow(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IdentifierError(String);

impl Display for IdentifierError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid identifier: {}", self.0)
    }
}

impl Error for IdentifierError {}
