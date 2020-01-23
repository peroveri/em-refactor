macro_rules! foo {
    () => {
        let i = 1;
    };
}
fn main() {
    let i = 0;
    foo!();
    assert_eq!(0, i);
}