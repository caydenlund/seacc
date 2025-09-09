#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    Identifier(String),
    LiteralInt(i64),
    LiteralFloat(u64), // Store as bits for Hash/Eq
    LiteralChar(u8),
    LiteralString(String),

    // Control keywords
    If,
    Else,
    For,
    While,
    Do,
    Return,
    Switch,
    Case,
    Break,
    Goto,

    // Type keywords
    Void,
    Unsigned,
    Char,
    Short,
    Int,
    Long,
    Float,
    Double,

    // Comparisons
    CmpLt,  // `<`
    CmpGt,  // `>`
    CmpLeq, // `<=`
    CmpGeq, // `>=`
    CmpEq,  // `==`
    CmpNeq, // `!=`

    // Arithmetic
    Incr, // `++`
    Decr, // `--`
    Star, // `*`
    Slash,
    Plus,
    Minus,
    LShift,
    RShift,
    Eq, // `=`
    StarEq,
    SlashEq,
    PlusEq,
    MinusEq,
    LShiftEq,
    RShiftEq,
    Ampersand,
    Semicolon,
    Colon,
    Comma,
    LParen,
    RParen,
    LSquare,
    RSquare,
    LBrace,
    RBrace,
}
