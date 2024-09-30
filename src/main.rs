fn main() {
    println!("Hello, world!");
}

#[test]
fn addition() {
    assert_eq!(process("3 + 6"), "9".to_string());
}
