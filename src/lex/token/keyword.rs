use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Struct,
    If,
    Else,
    For,
    Do,
    While,
    Return,
    Continue,
    Break,
    Switch,
}

impl FromStr for Keyword {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "struct" => Ok(Self::Struct),
            "if" => Ok(Self::If),
            "else" => Ok(Self::Else),
            "for" => Ok(Self::For),
            "do" => Ok(Self::Do),
            "while" => Ok(Self::While),
            "return" => Ok(Self::Return),
            "continue" => Ok(Self::Continue),
            "break" => Ok(Self::Break),
            "switch" => Ok(Self::Switch),
            _ => Err(()),
        }
    }
}
