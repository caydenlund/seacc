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
    /// Creates a new `Keyword` with variant selected based on text
    ///
    /// # Errors
    /// Returns an error if `raw_keyword` is not a recognized C keyword.
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
        match self {
            Self::Auto => write!(f, "auto"),
            Self::Break => write!(f, "break"),
            Self::Case => write!(f, "case"),
            Self::Char => write!(f, "char"),
            Self::Const => write!(f, "const"),
            Self::Continue => write!(f, "continue"),
            Self::Default => write!(f, "default"),
            Self::Do => write!(f, "do"),
            Self::Double => write!(f, "double"),
            Self::Else => write!(f, "else"),
            Self::Enum => write!(f, "enum"),
            Self::Extern => write!(f, "extern"),
            Self::Float => write!(f, "float"),
            Self::For => write!(f, "for"),
            Self::Goto => write!(f, "goto"),
            Self::If => write!(f, "if"),
            Self::Inline => write!(f, "inline"),
            Self::Int => write!(f, "int"),
            Self::Long => write!(f, "long"),
            Self::Register => write!(f, "register"),
            Self::Restrict => write!(f, "restrict"),
            Self::Return => write!(f, "return"),
            Self::Short => write!(f, "short"),
            Self::Signed => write!(f, "signed"),
            Self::Sizeof => write!(f, "sizeof"),
            Self::Static => write!(f, "static"),
            Self::Struct => write!(f, "struct"),
            Self::Switch => write!(f, "switch"),
            Self::Typedef => write!(f, "typedef"),
            Self::Union => write!(f, "union"),
            Self::Unsigned => write!(f, "unsigned"),
            Self::Void => write!(f, "void"),
            Self::Volatile => write!(f, "volatile"),
            Self::While => write!(f, "while"),
            Self::Alignas => write!(f, "_Alignas"),
            Self::Alignof => write!(f, "_Alignof"),
            Self::Atomic => write!(f, "_Atomic"),
            Self::Bool => write!(f, "_Bool"),
            Self::Complex => write!(f, "_Complex"),
            Self::Generic => write!(f, "_Generic"),
            Self::Imaginary => write!(f, "_Imaginary"),
            Self::Noreturn => write!(f, "_Noreturn"),
            Self::StaticAssert => write!(f, "_Static_assert"),
            Self::ThreadLocal => write!(f, "_Thread_local"),
        }
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
