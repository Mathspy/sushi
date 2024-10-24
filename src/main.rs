fn main() {
    println!("Hello, world!");
}

struct Math {
    init: Vec<Init>,
    end_expression: Expr,
}

enum Init {
    Var(Var),
    UtilityUse(UtilityUse),
}

struct Var {
    name: String,
    value: Expr,
}

struct UtilityUse {
    ident: String,
    param: Expr,
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
    let (output, final_output) = redacted_name(context, math);
    format!("{output}{final_output}")
}

fn parse(input: &str) -> Math {
    use chumsky::Parser;

    parser().parse(input).unwrap()
}

mod context {
    use crate::redacted_name_expr;

    use super::Expr;
    use std::{cell::RefCell, collections::HashMap, rc::Rc};

    pub(crate) enum Identifier {
        Expr(Expr),
        Utility(RefCell<Box<dyn FnMut(Option<i32>) -> Expr>>),
    }

    pub(crate) struct Context {
        identifiers: HashMap<String, Identifier>,
        output: Rc<RefCell<String>>,
    }

    impl Context {
        pub(crate) fn builtins(seed: u64) -> Self {
            let mut rng = oorandom::Rand32::new(seed);
            let output = Rc::new(RefCell::new(String::new()));
            Context {
                identifiers: HashMap::from([
                    (
                        "RAND".to_string(),
                        Identifier::Utility(RefCell::new(Box::new(move |_| {
                            Expr::Number(rng.rand_range(0..100) as i32)
                        }))),
                    ),
                    (
                        "PRINT".to_string(),
                        Identifier::Utility(RefCell::new(Box::new({
                            use std::fmt::Write;

                            let output = Rc::clone(&output);

                            move |input| {
                                let _ = writeln!(
                                    output.borrow_mut(),
                                    "{}",
                                    &input.expect("print requires an expression")
                                );

                                // TODO: Hacky
                                Expr::Number(0)
                            }
                        }))),
                    ),
                ]),
                calculated: RefCell::default(),
                output,
            }
        }

        pub(crate) fn with_variables<I>(mut self, variables: I) -> Self
        where
            I: IntoIterator<Item = (String, Expr)>,
        {
            self.identifiers.extend(
                variables
                    .into_iter()
                    .map(|(id, expr)| (id, Identifier::Expr(expr))),
            );

            self
        }

        pub(crate) fn redacted_name_identifier(&self, id: &str, input: Option<i32>) -> i32 {
            match self.identifiers.get(id) {
                Some(Identifier::Expr(expr)) => redacted_name_expr(self, expr),
                Some(Identifier::Utility(util)) => {
                    redacted_name_expr(self, &util.borrow_mut()(input))
                }
                None => panic!("unknown identifier {id}"),
            }
        }

        pub fn into_output(self) -> String {
            drop(self.identifiers);
            Rc::into_inner(self.output)
                .expect("all references to be dropped")
                .into_inner()
        }
    }
}

fn redacted_name_expr(cx: &context::Context, expr: &Expr) -> i32 {
    match expr {
        Expr::Number(a) => *a,
        Expr::Ident(ident) => cx.redacted_name_identifier(ident, None),
        Expr::Negate(a) => -redacted_name_expr(cx, a),
        Expr::Add(a, b) => redacted_name_expr(cx, a) + redacted_name_expr(cx, b),
        Expr::Subtract(a, b) => redacted_name_expr(cx, a) - redacted_name_expr(cx, b),
        Expr::Multiply(a, b) => redacted_name_expr(cx, a) * redacted_name_expr(cx, b),
        Expr::Divide(a, b) => redacted_name_expr(cx, a) / redacted_name_expr(cx, b),
    }
}

// This is named like that to not ruin the surprise for my friend who is working on this challenge
// too
fn redacted_name(cx: context::Context, math: Math) -> (String, i32) {
    let (variables, utility_uses) = math.init.into_iter().fold(
        (Vec::new(), Vec::new()),
        |(mut variables, mut utility_uses), init| {
            match init {
                Init::Var(var) => variables.push((var.name, var.value)),
                Init::UtilityUse(utility_use) => utility_uses.push(utility_use),
            };

            (variables, utility_uses)
        },
    );

    let cx = cx.with_variables(variables);

    utility_uses.into_iter().for_each(|utility_use| {
        cx.redacted_name_identifier(
            &utility_use.ident,
            Some(redacted_name_expr(&cx, &utility_use.param)),
        );
    });

    let final_output = redacted_name_expr(&cx, &math.end_expression);

    (cx.into_output(), final_output)
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

    let var = text::ident()
        .then_ignore(equal)
        .then(expr.clone())
        .padded()
        .map(|(name, value)| Var { name, value });

    let utility_use = text::ident()
        .then(expr.clone().padded())
        .map(|(ident, param)| UtilityUse { ident, param });

    let init = utility_use
        .map(Init::UtilityUse)
        .or(var.map(Init::Var))
        .padded()
        .repeated();

    init.then(expr)
        .map(|(init, end_expression)| Math {
            init,
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

#[test]
fn simple_print() {
    assert_eq!(
        process(
            "
    a = 5 + 3
    PRINT a

    a
    "
        ),
        "8\n8".to_string()
    );
}
