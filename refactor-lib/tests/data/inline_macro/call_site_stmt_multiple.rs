macro_rules! foo {
    () => { let i = 1; let j = 2; };
}
fn main() {
    foo!();
}