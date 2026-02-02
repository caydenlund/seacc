use std::{error::Error, fmt::Display};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Keyword {
    Auto,         // `auto`
    Break,        // `break`
    Case,         // `case`
    Char,         // `char`
    Const,        // `const`
    Continue,     // `continue`
    Default,      // `default`
    Do,           // `do`
    Double,       // `double`
    Else,         // `else`
    Enum,         // `enum`
    Extern,       // `extern`
    Float,        // `float`
    For,          // `for`
    Goto,         // `goto`
    If,           // `if`
    Inline,       // `inline`
    Int,          // `int`
    Long,         // `long`
    Register,     // `register`
    Restrict,     // `restrict`
    Return,       // `return`
    Short,        // `short`
    Signed,       // `signed`
    Sizeof,       // `sizeof`
    Static,       // `static`
    Struct,       // `struct`
    Switch,       // `switch`
    Typedef,      // `typedef`
    Union,        // `union`
    Unsigned,     // `unsigned`
    Void,         // `void`
    Volatile,     // `volatile`
    While,        // `while`
    Alignas,      // `_Alignas`
    Alignof,      // `_Alignof`
    Atomic,       // `_Atomic`
    Bool,         // `_Bool`
    Complex,      // `_Complex`
    Generic,      // `_Generic`
    Imaginary,    // `_Imaginary`
    Noreturn,     // `_Noreturn`
    StaticAssert, // `_Static_assert`
    ThreadLocal,  // `_Thread_local`
}

impl Keyword {
    pub fn new(raw_keyword: &str) -> Result<Self, KeywordError> {
        match raw_keyword {
            "auto" => Ok(Self::Auto),
            "break" => Ok(Self::Break),
            "case" => Ok(Self::Case),
            "char" => Ok(Self::Char),
            "const" => Ok(Self::Const),
            "continue" => Ok(Self::Continue),
            "default" => Ok(Self::Default),
            "do" => Ok(Self::Do),
            "double" => Ok(Self::Double),
            "else" => Ok(Self::Else),
            "enum" => Ok(Self::Enum),
            "extern" => Ok(Self::Extern),
            "float" => Ok(Self::Float),
            "for" => Ok(Self::For),
            "goto" => Ok(Self::Goto),
            "if" => Ok(Self::If),
            "inline" => Ok(Self::Inline),
            "int" => Ok(Self::Int),
            "long" => Ok(Self::Long),
            "register" => Ok(Self::Register),
            "restrict" => Ok(Self::Restrict),
            "return" => Ok(Self::Return),
            "short" => Ok(Self::Short),
            "signed" => Ok(Self::Signed),
            "sizeof" => Ok(Self::Sizeof),
            "static" => Ok(Self::Static),
            "struct" => Ok(Self::Struct),
            "switch" => Ok(Self::Switch),
            "typedef" => Ok(Self::Typedef),
            "union" => Ok(Self::Union),
            "unsigned" => Ok(Self::Unsigned),
            "void" => Ok(Self::Void),
            "volatile" => Ok(Self::Volatile),
            "while" => Ok(Self::While),
            "_Alignas" => Ok(Self::Alignas),
            "_Alignof" => Ok(Self::Alignof),
            "_Atomic" => Ok(Self::Atomic),
            "_Bool" => Ok(Self::Bool),
            "_Complex" => Ok(Self::Complex),
            "_Generic" => Ok(Self::Generic),
            "_Imaginary" => Ok(Self::Imaginary),
            "_Noreturn" => Ok(Self::Noreturn),
            "_Static_assert" => Ok(Self::StaticAssert),
            "_Thread_local" => Ok(Self::ThreadLocal),
            _ => Err(KeywordError(raw_keyword.into())),
        }
    }
}

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeywordError(String);

impl Display for KeywordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid keyword: {}", self.0)
    }
}

impl Error for KeywordError {}
