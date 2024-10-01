fn main() {
    println!("Hello, world!");
}

enum Expr {
    Number(i32),
    Inverse(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Subtract(Box<Expr>, Box<Expr>),
    Multiply(Box<Expr>, Box<Expr>),
    Divide(Box<Expr>, Box<Expr>),
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
        Expr::Inverse(a) => -redacted_name(a),
        Expr::Add(a, b) => redacted_name(a) + redacted_name(b),
        Expr::Subtract(a, b) => redacted_name(a) - redacted_name(b),
        Expr::Multiply(a, b) => redacted_name(a) * redacted_name(b),
        Expr::Divide(a, b) => redacted_name(a) / redacted_name(b),
    }
}

fn parser() -> impl chumsky::Parser<char, Expr, Error = chumsky::error::Simple<char>> {
    use chumsky::{primitive::just, text, text::TextParser, Parser};

    let plus = just('+').padded();
    let minus = just('-').padded();
    let star = just('*').padded();
    let slash = just('/').padded();

    let positive_int = text::int(10)
        .map(|s: String| s.parse::<i32>().unwrap())
        .map(Expr::Number);

    let int = minus
        .padded()
        .repeated()
        .then(positive_int)
        .foldr(|_minus, expr| Expr::Inverse(Box::new(expr)));

    let prod_and_div = int
        .then(
            star.to(Expr::Multiply as fn(_, _) -> _)
                .or(slash.to(Expr::Divide as fn(_, _) -> _))
                .then(int)
                .repeated(),
        )
        .foldl(|a, (operation, b)| operation(Box::new(a), Box::new(b)));

    prod_and_div
        .then(
            plus.to(Expr::Add as fn(_, _) -> _)
                .or(minus.to(Expr::Subtract as fn(_, _) -> _))
                .then(prod_and_div)
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

#[test]
fn multiple() {
    assert_eq!(process("3 * 6"), "18".to_string());
    assert_eq!(process("3*6"), "18".to_string());
    assert_eq!(process("3*- 6"), "-18".to_string());
}

#[test]
fn divide() {
    assert_eq!(process("6 / 3"), "2".to_string());
    assert_eq!(process("6/3"), "2".to_string());
    assert_eq!(process("6/-3"), "-2".to_string());
    assert_eq!(process("-6/- 3"), "2".to_string());
}

#[test]
fn order_of_operations() {
    assert_eq!(process("1 + 2 * 3"), "7".to_string());
}
