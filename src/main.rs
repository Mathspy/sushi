fn main() {
    println!("Hello, world!");
}

enum Expr {
    Number(i32),
    Negate(Box<Expr>),
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
        Expr::Negate(a) => -redacted_name(a),
        Expr::Add(a, b) => redacted_name(a) + redacted_name(b),
        Expr::Subtract(a, b) => redacted_name(a) - redacted_name(b),
        Expr::Multiply(a, b) => redacted_name(a) * redacted_name(b),
        Expr::Divide(a, b) => redacted_name(a) / redacted_name(b),
    }
}

fn parser() -> impl chumsky::Parser<char, Expr, Error = chumsky::error::Simple<char>> {
    use chumsky::{
        primitive::{end, just},
        recursive::recursive,
        text::{self, TextParser},
        Parser,
    };

    let plus = just('+').padded();
    let minus = just('-').padded();
    let star = just('*').padded();
    let slash = just('/').padded();
    let open_paren = just('(').padded();
    let close_paren = just(')').padded();

    recursive(|expr| {
        let positive_int = text::int(10)
            .map(|s: String| s.parse::<i32>().unwrap())
            .map(Expr::Number);
        let paren_wrapped_expr = expr.clone().delimited_by(open_paren, close_paren);

        let unit = minus
            .padded()
            .repeated()
            .then(positive_int.or(paren_wrapped_expr))
            .foldr(|_minus, expr| Expr::Negate(Box::new(expr)));

        let prod_and_div = unit
            .clone()
            .then(
                star.to(Expr::Multiply as fn(_, _) -> _)
                    .or(slash.to(Expr::Divide as fn(_, _) -> _))
                    .then(unit)
                    .repeated(),
            )
            .foldl(|a, (operation, b)| operation(Box::new(a), Box::new(b)));

        prod_and_div
            .clone()
            .then(
                plus.to(Expr::Add as fn(_, _) -> _)
                    .or(minus.to(Expr::Subtract as fn(_, _) -> _))
                    .then(prod_and_div)
                    .repeated(),
            )
            .foldl(|a, (operation, b)| operation(Box::new(a), Box::new(b)))
    })
    .then_ignore(end())
}

#[test]
fn addition() {
    assert_eq!(process("3 + 6"), "9".to_string());
    assert_eq!(process("3+6"), "9".to_string());
    assert_eq!(process("   3 + 6   "), "9".to_string());
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
    assert_eq!(process("(1 + 2) * 3"), "9".to_string());
    assert_eq!(process(" ( 1 + 2) * 3"), "9".to_string());
}

#[test]
fn milestone_1() {
    assert_eq!(process("5 + (3 + 7) * 99"), "995".to_string());
}
