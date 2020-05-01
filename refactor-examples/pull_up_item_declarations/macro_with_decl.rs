macro_rules! foo {
    () => {
        fn bar () { }
    }
}
fn main() {
    foo!();
}