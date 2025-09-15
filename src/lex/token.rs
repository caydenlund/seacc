//! lex/token: Definition of the [`TokenType`] and [`Token`] datatypes for C programs

/// Token types for C language lexical analysis
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    /// Variable or function identifier
    Identifier(String),
    /// Integer literal value
    LiteralInt(i64),
    /// Floating-point literal stored as raw bits for Hash/Eq compatibility
    LiteralFloat(u64),
    /// Character literal (single byte)
    LiteralChar(u8),
    /// String literal
    LiteralString(String),

    // Control flow keywords
    /// `if` keyword
    If,
    /// `else` keyword
    Else,
    /// `for` keyword
    For,
    /// `while` keyword
    While,
    /// `do` keyword
    Do,
    /// `return` keyword
    Return,
    /// `switch` keyword
    Switch,
    /// `case` keyword
    Case,
    /// `break` keyword
    Break,
    /// `goto` keyword
    Goto,

    // Type keywords
    /// `void` type keyword
    Void,
    /// `unsigned` type modifier
    Unsigned,
    /// `char` type keyword
    Char,
    /// `short` type keyword
    Short,
    /// `int` type keyword
    Int,
    /// `long` type keyword
    Long,
    /// `float` type keyword
    Float,
    /// `double` type keyword
    Double,

    // Comparison operators
    /// Less than operator `<`
    CmpLt,
    /// Greater than operator `>`
    CmpGt,
    /// Less than or equal operator `<=`
    CmpLeq,
    /// Greater than or equal operator `>=`
    CmpGeq,
    /// Equality operator `==`
    CmpEq,
    /// Not equal operator `!=`
    CmpNeq,

    // Arithmetic and assignment operators
    /// Increment operator `++`
    Incr,
    /// Decrement operator `--`
    Decr,
    /// Multiplication operator or dereference `*`
    Star,
    /// Division operator `/`
    Slash,
    /// Addition operator `+`
    Plus,
    /// Subtraction operator `-`
    Minus,
    /// Left shift operator `<<`
    LShift,
    /// Right shift operator `>>`
    RShift,
    /// Assignment operator `=`
    Eq,
    /// Multiplication assignment `*=`
    StarEq,
    /// Division assignment `/=`
    SlashEq,
    /// Addition assignment `+=`
    PlusEq,
    /// Subtraction assignment `-=`
    MinusEq,
    /// Left shift assignment `<<=`
    LShiftEq,
    /// Right shift assignment `>>=`
    RShiftEq,
    /// Bitwise AND operator or address-of `&`
    Ampersand,

    // Punctuation
    /// Semicolon `;`
    Semicolon,
    /// Colon `:`
    Colon,
    /// Comma `,`
    Comma,
    /// Left parenthesis `(`
    LParen,
    /// Right parenthesis `)`
    RParen,
    /// Left square bracket `[`
    LSquare,
    /// Right square bracket `]`
    RSquare,
    /// Left brace `{`
    LBrace,
    /// Right brace `}`
    RBrace,
}
