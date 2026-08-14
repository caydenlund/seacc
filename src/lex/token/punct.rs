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
    Arrow,     // `->`

    Star,      // `*`
    Slash,     // `/`
    Percent,   // `%`
    Plus,      // `+`
    Minus,     // `-`
    Amp,       // `&`
    Caret,     // `^`
    Pipe,      // `|`
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
    AmpEq,     // `&=`
    CaretEq,   // `^=`
    PipeEq,    // `|=`

    Bang,       // `!`
    Tilde,      // `~`
    PlusPlus,   // `++`
    MinusMinus, // `--`
    Question,   // `?`
    Colon,      // `:`

    EqEq,     // `==`
    BangEq,   // `!=`
    Gt,       // `>`
    GtEq,     // `>=`
    Lt,       // `<`
    LtEq,     // `<=`
    AmpAmp,   // `&&`
    PipePipe, // `||`
}
