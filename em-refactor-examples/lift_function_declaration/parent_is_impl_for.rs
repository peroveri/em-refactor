fn main() { }

struct S;
trait T {
    fn foo() {}
}
impl T for S {
    fn foo() {
        fn bar() { }
        bar();
    }
}