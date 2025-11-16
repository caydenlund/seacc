use seacc::lex::Lexer;

fn main() {
    let lexer = Lexer::build();
    let source = "int main() { return 42; }";
    
    match lexer.lex(source) {
        Ok(tokens) => {
            for token in tokens {
                println!("{:?}", token);
            }
        }
        Err(error) => {
            println!("Lexical error: {:?}", error);
        }
    }
}