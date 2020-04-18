macro_rules! foo {
    () => {
        let i = 1;
    };
}
fn main() {
    let i = 0;
    let i_foo0 = 1;
    assert_eq!(0, i);
}