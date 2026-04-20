use seacc::pp_lexer::PpLexer;
use seacc::preprocessor::Preprocessor;
use seacc::source_reader::SourceReader;
use seacc::token::{HeaderNameType, PreprocessingToken, Punctuator, StringLiteral};
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::process;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        process::exit(1);
    }

    let filename = &args[1];
    let file = File::open(filename)?;
    let buf_reader = BufReader::new(file);

    // Create the processing pipeline: SourceReader -> PpLexer -> Preprocessor
    let source_reader = SourceReader::new(filename, buf_reader);
    let pp_lexer = PpLexer::new(source_reader);
    let preprocessor = Preprocessor::new(pp_lexer, filename);

    // Process and output the expanded source
    for result in preprocessor {
        match result {
            Ok(spanned_token) => {
                print_token(&spanned_token.value);
            }
            Err(e) => {
                eprintln!("Preprocessor error: {:?}", e);
                process::exit(1);
            }
        }
    }

    Ok(())
}

fn print_token(token: &PreprocessingToken) {
    match token {
        PreprocessingToken::HeaderName(header_type, name) => match header_type {
            HeaderNameType::Angled => print!("<{name}>"),
            HeaderNameType::Quoted => print!("\"{name}\""),
        },
        PreprocessingToken::Identifier(id) => print!("{}", id.as_ref()),
        PreprocessingToken::PpNumber(num) => print!("{num}"),
        PreprocessingToken::CharacterConstant(ch) => print!("'{}'", char::from(*ch)),
        PreprocessingToken::StringLiteral(StringLiteral(_, contents)) => print!("\"{contents}\""),
        PreprocessingToken::Punctuator(p) => print!("{}", punctuator_to_str(p)),
        PreprocessingToken::Whitespace => print!(" "),
        PreprocessingToken::Newline => println!(),
        PreprocessingToken::OtherChar(ch) => print!("{ch}"),
    }
}

fn punctuator_to_str(p: &Punctuator) -> &'static str {
    use Punctuator::*;
    match p {
        LBracket => "[",
        RBracket => "]",
        LParen => "(",
        RParen => ")",
        LCurly => "{",
        RCurly => "}",
        Dot => ".",
        Arrow => "->",
        Incr => "++",
        Decr => "--",
        Amp => "&",
        Asterisk => "*",
        Plus => "+",
        Minus => "-",
        Tilde => "~",
        Exclamation => "!",
        Slash => "/",
        Percent => "%",
        LShift => "<<",
        RShift => ">>",
        Lt => "<",
        Gt => ">",
        Leq => "<=",
        Geq => ">=",
        Eq => "==",
        Neq => "!=",
        BitXor => "^",
        BitOr => "|",
        And => "&&",
        Or => "||",
        Question => "?",
        Colon => ":",
        Semicolon => ";",
        Ellips => "...",
        Assign => "=",
        StarAssign => "*=",
        SlashAssign => "/=",
        PercentAssign => "%=",
        PlusAssign => "+=",
        MinusAssign => "-=",
        LShiftAssign => "<<=",
        RShiftAssign => ">>=",
        BitAndAssign => "&=",
        BitXorAssign => "^=",
        BitOrAssign => "|=",
        Comma => ",",
        Hash => "#",
        HashHash => "##",
    }
}
