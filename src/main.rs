fn main() {
    println!("Hello, world!");
}

struct Math {
    variables: Vec<Var>,
    end_expression: Expr,
}

struct Var {
    name: String,
    value: Expr,
}

#[derive(Clone)]
enum Expr {
    Number(i32),
    Ident(String),
    Negate(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Subtract(Box<Expr>, Box<Expr>),
    Multiply(Box<Expr>, Box<Expr>),
    Divide(Box<Expr>, Box<Expr>),
}

pub fn process(input: &str) -> String {
    let mut seed = [0u8; 8];
    getrandom::getrandom(&mut seed).expect("OS to seed us");
    process_with_seed(input, u64::from_be_bytes(seed))
}

pub fn process_with_seed(input: &str, seed: u64) -> String {
    let math = parse(input);
    let context = context::Context::builtins(seed);
    let output = redacted_name(context, math);
    output.to_string()
}

fn parse(input: &str) -> Math {
    use chumsky::Parser;

    parser().parse(input).unwrap()
}

mod context {
    use super::Expr;
    use std::{cell::RefCell, collections::HashMap};

    pub(crate) enum Identifier {
        Expr(Expr),
        Utility(RefCell<Box<dyn FnMut() -> Expr>>),
    }

    pub(crate) struct Context {
        identifiers: HashMap<String, Identifier>,
    }

    impl Context {
        pub(crate) fn builtins(seed: u64) -> Self {
            let mut rng = oorandom::Rand32::new(seed);
            Context {
                identifiers: HashMap::from([(
                    "RAND".to_string(),
                    Identifier::Utility(RefCell::new(Box::new(move || {
                        Expr::Number(rng.rand_range(0..100) as i32)
                    }))),
                )]),
            }
        }

        pub(crate) fn with_variables<I>(self, variables: I) -> Self
        where
            I: IntoIterator<Item = (String, Expr)>,
        {
            let mut identifiers = self.identifiers;
            identifiers.extend(
                variables
                    .into_iter()
                    .map(|(id, expr)| (id, Identifier::Expr(expr))),
            );

            Context { identifiers }
        }

        pub(crate) fn get(&self, id: &str) -> Expr {
            match self.identifiers.get(id) {
                Some(Identifier::Expr(expr)) => expr.clone(),
                Some(Identifier::Utility(util)) => util.borrow_mut()(),
                None => panic!("unknown identifier {id}"),
            }
        }
    }
}

fn redacted_name_expr(cx: &context::Context, expr: &Expr) -> i32 {
    match expr {
        Expr::Number(a) => *a,
        Expr::Ident(ident) => redacted_name_expr(cx, &cx.get(ident)),
        Expr::Negate(a) => -redacted_name_expr(cx, a),
        Expr::Add(a, b) => redacted_name_expr(cx, a) + redacted_name_expr(cx, b),
        Expr::Subtract(a, b) => redacted_name_expr(cx, a) - redacted_name_expr(cx, b),
        Expr::Multiply(a, b) => redacted_name_expr(cx, a) * redacted_name_expr(cx, b),
        Expr::Divide(a, b) => redacted_name_expr(cx, a) / redacted_name_expr(cx, b),
    }
}

// This is named like that to not ruin the surprise for my friend who is working on this challenge
// too
fn redacted_name(cx: context::Context, math: Math) -> i32 {
    let variables = math.variables.into_iter().map(|var| (var.name, var.value));
    let cx = cx.with_variables(variables);

    redacted_name_expr(&cx, &math.end_expression)
}

fn parser() -> impl chumsky::Parser<char, Math, Error = chumsky::error::Simple<char>> {
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
    let equal = just('=').padded();

    let ident = text::ident().map(Expr::Ident);

    let expr = recursive(|expr| {
        let positive_int = text::int(10)
            .map(|s: String| s.parse::<i32>().unwrap())
            .map(Expr::Number);
        let paren_wrapped_expr = expr.clone().delimited_by(open_paren, close_paren);

        let unit = minus
            .padded()
            .repeated()
            .then(positive_int.or(paren_wrapped_expr).or(ident))
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
    });

    let vars = text::ident()
        .then_ignore(equal)
        .then(expr.clone())
        .padded()
        .map(|(name, value)| Var { name, value })
        .repeated();

    vars.then(expr)
        .map(|(variables, end_expression)| Math {
            variables,
            end_expression,
        })
        .padded()
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

#[test]
fn simple_variable() {
    assert_eq!(
        process(
            "
    a = 5 + 3

    a
    "
        ),
        "8".to_string()
    );
}

#[test]
fn milestone_2() {
    assert_eq!(
        process(
            "
    a = 5 + 3
    b = 89

    b / a
    "
        ),
        "11".to_string()
    );
}

#[test]
fn milestone_3() {
    assert_eq!(
        process_with_seed(
            "
    a = RAND
    b = RAND

    a + b
    ",
            42
        ),
        "55".to_string()
    );
}
