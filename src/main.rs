fn main() {
    println!("Hello, world!");
}

enum Expr {
    Add(i32, i32),
}

pub fn process(input: &str) -> String {
    let expr = parse(input);
    let output = redacted_name(expr);
    output.to_string()
}

fn parse(input: &str) -> Expr {
    use chumsky::Parser;

    parser().parse(input).unwrap()
}

// This is named like that to not ruin the surprise for my friend who is working on this challenge
// too
fn redacted_name(expr: Expr) -> i32 {
    match expr {
        Expr::Add(a, b) => a + b,
    }
}

fn parser() -> impl chumsky::Parser<char, Expr, Error = chumsky::error::Simple<char>> {
    use chumsky::{primitive::just, text, text::TextParser, Parser};

    let int = text::int(10).map(|s: String| s.parse().unwrap());
    let plus = just('+').padded();

    int.then_ignore(plus)
        .then(int)
        .map(|(a, b)| Expr::Add(a, b))
}

#[test]
fn addition() {
    assert_eq!(process("3 + 6"), "9".to_string());
}
