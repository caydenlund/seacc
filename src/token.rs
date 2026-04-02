mod constant;
mod identifier;
mod keyword;
mod string;

pub use constant::*;
pub use identifier::*;
pub use keyword::*;
pub use string::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Keyword(Keyword),
    Identifier(Identifier),
    Constant(Constant),
    StringLiteral(StringLiteral),
    Punctuator(Punctuator),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreprocessingToken {
    HeaderName(HeaderNameType, String),
    Identifier(Identifier),
    PpNumber(String),
    CharacterConstant(CharacterConstant),
    StringLiteral(StringLiteral),
    Punctuator(Punctuator),
    Whitespace,
    Newline,
    OtherChar(char),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum HeaderNameType {
    Angled,
    Quoted,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Punctuator {
    LBracket,      // `[` or `<:`
    RBracket,      // `]` or `:>`
    LParen,        // `(`
    RParen,        // `)`
    LCurly,        // `{` or `<%`
    RCurly,        // `}` or `%>`
    Dot,           // `.`
    Arrow,         // `->`
    Incr,          // `++`
    Decr,          // `--`
    Amp,           // `&`
    Asterisk,      // `*`
    Plus,          // `+`
    Minus,         // `-`
    Tilde,         // `~`
    Exclamation,   // `!`
    Slash,         // `/`
    Percent,       // `%`
    LShift,        // `<<`
    RShift,        // `>>`
    Lt,            // `<`
    Gt,            // `>`
    Leq,           // `<=`
    Geq,           // `>=`
    Eq,            // `==`
    Neq,           // `!=`
    BitXor,        // `^`
    BitOr,         // `|`
    And,           // `&&`
    Or,            // `||`
    Question,      // `?`
    Colon,         // `:`
    Semicolon,     // `;`
    Ellips,        // `...`
    Assign,        // `=`
    StarAssign,    // `*=`
    SlashAssign,   // `/=`
    PercentAssign, // `%=`
    PlusAssign,    // `+=`
    MinusAssign,   // `-=`
    LShiftAssign,  // `<<=`
    RShiftAssign,  // `>>=`
    BitAndAssign,  // `&=`
    BitXorAssign,  // `^=`
    BitOrAssign,   // `|=`
    Comma,         // `,`
    Hash,          // `#` or `%:`
    HashHash,      // `##` or `%:%:`
}
