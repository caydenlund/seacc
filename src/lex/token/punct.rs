#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Punct {
    Lparen,    // `(`
    Rparen,    // `)`
    Lsquare,   // `[`
    Rsquare,   // `]`
    Lcurly,    // `{`
    Rcurly,    // `}`
    Comma,     // `,`
    Semicolon, // `;`
    Dot,       // `.`

    Star,      // `*`
    Slash,     // `/`
    Percent,   // `%`
    Plus,      // `+`
    Minus,     // `-`
    Lshift,    // `<<`
    Rshift,    // `>>`
    Eq,        // `=`
    StarEq,    // `*=`
    SlashEq,   // `/=`
    PercentEq, // `%=`
    PlusEq,    // `+=`
    MinusEq,   // `-=`
    LshiftEq,  // `<<=`
    RshiftEq,  // `>>=`
    Bang,      // `!`

    EqEq,   // `==`
    BangEq, // `!=`
    Gt,     // `>`
    GtEq,   // `>=`
    Lt,     // `<`
    LtEq,   // `<=`
    And,    // `&&`
    Or,     // `||`
}
