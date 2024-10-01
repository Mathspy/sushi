fn main() {
    println!("Hello, world!");
}

enum Expr {
    Number(i32),
    Add(Box<Expr>, Box<Expr>),
    Subtract(Box<Expr>, Box<Expr>),
}

pub fn process(input: &str) -> String {
    let expr = parse(input);
    let output = redacted_name(&expr);
    output.to_string()
}

fn parse(input: &str) -> Expr {
    use chumsky::Parser;

    parser().parse(input).unwrap()
}

// This is named like that to not ruin the surprise for my friend who is working on this challenge
// too
fn redacted_name(expr: &Expr) -> i32 {
    match expr {
        Expr::Number(a) => *a,
        Expr::Add(a, b) => redacted_name(a) + redacted_name(b),
        Expr::Subtract(a, b) => redacted_name(a) - redacted_name(b),
    }
}

fn parser() -> impl chumsky::Parser<char, Expr, Error = chumsky::error::Simple<char>> {
    use chumsky::{primitive::just, text, text::TextParser, Parser};

    let positive_int = text::int(10).map(|s: String| s.parse::<i32>().unwrap());
    let plus = just('+').padded();
    let minus = just('-').padded();

    let int = minus
        .padded()
        .or_not()
        .map(|minus| minus.is_some())
        .then(positive_int)
        .map(|(is_negative, int)| if is_negative { -int } else { int })
        .map(Expr::Number);

    int.then(
        plus.to(Expr::Add as fn(_, _) -> _)
            .or(minus.to(Expr::Subtract as fn(_, _) -> _))
            .then(int)
            .repeated(),
    )
    .foldl(|a, (operation, b)| operation(Box::new(a), Box::new(b)))
}

#[test]
fn addition() {
    assert_eq!(process("3 + 6"), "9".to_string());
    assert_eq!(process("3+6"), "9".to_string());
}

#[test]
fn subtraction() {
    assert_eq!(process("3 - 6"), "-3".to_string());
    assert_eq!(process("3-6"), "-3".to_string());
}

#[test]
fn negative() {
    assert_eq!(process("- 6"), "-6".to_string());
    assert_eq!(process("-6"), "-6".to_string());
    assert_eq!(process("--6"), "6".to_string());
    assert_eq!(process("---6"), "-6".to_string());
}

#[test]
fn complex_add_sub() {
    assert_eq!(process("5 + -3 - 51 - - 7 + 0"), "-42".to_string());
}
