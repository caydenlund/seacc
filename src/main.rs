use seacc::lex::{RegexComponent, RegexPattern};

fn main() {
    // Create an interesting regex pattern: (a|b)+abb
    let pattern = vec![
        RegexComponent::Repeat {
            item: Box::new(RegexComponent::Alternation(
                vec![RegexComponent::Literal(vec![b'a'])],
                vec![RegexComponent::Literal(vec![b'b'])],
            )),
            min: 1,
            max: None,
        },
        RegexComponent::Literal(vec![b'a']),
        RegexComponent::Literal(vec![b'b']),
        RegexComponent::Literal(vec![b'b']),
    ];

    let regex = RegexPattern::new(pattern);
    let nfa = regex.to_nfa();

    println!("// Regex pattern: (a|b)+abb");
    println!("{}", nfa.to_dfa().to_dot());
}
