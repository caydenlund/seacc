use seacc::lex::{Nfa, RegexComponent, RegexPattern, TokenType};

fn main() {
    let patterns_and_tokens = [
        (
            vec![RegexComponent::Literal("if".bytes().collect())],
            TokenType::If,
        ),
        (
            vec![RegexComponent::Literal("else".bytes().collect())],
            TokenType::Else,
        ),
        (
            vec![RegexComponent::Literal("for".bytes().collect())],
            TokenType::For,
        ),
        (
            vec![RegexComponent::Repeat {
                item: Box::new(RegexComponent::char_range(b'a', b'z')),
                min: 1,
                max: None,
            }],
            TokenType::Identifier("generic".to_string()),
        ),
    ];

    let nfas: Vec<Nfa> = patterns_and_tokens
        .iter()
        .map(|(pat, token)| RegexPattern::new(pat).to_nfa(token.clone()))
        .collect();

    let nfa = Nfa::merge(&nfas);

    println!("{}", nfa.to_dfa().minimize().to_dot());
}
