fn bar() -> i32 {
    {
        let foo = || { 1 };
        foo()
    }
}
fn main() {
}