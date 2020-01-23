macro_rules! foo {
    () => { foo!(1); };
    (1) => { };
}
fn main() {
    foo!();
}
// Inlining inside a macro is not supported