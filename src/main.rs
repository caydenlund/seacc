use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Clone, Parser)]
struct CliArgs {
    /// The source files to compile
    #[arg(required = true)]
    sources: Vec<PathBuf>,
    /// Dump the lexed tokens to stdout
    #[arg(long, default_value_t = false)]
    output_tokens: bool,
}

fn main() {
    let args = CliArgs::parse();

    let lexer = seacc::lex::Lexer::build();

    for source_file in args.sources {
        let Ok(source) = std::fs::read_to_string(&source_file) else {
            panic!(
                "Unable to open file '{}' for reading",
                source_file.display()
            );
        };
        let tokens = lexer
            .lex(&source)
            .unwrap_or_else(|e| panic!("Lex error: '{e:?}'"));
        if args.output_tokens {
            tokens.iter().for_each(|t| println!("{t:?}"));
        }
    }
}
