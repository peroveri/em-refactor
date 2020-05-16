fn main() { }

struct S;
trait T {
    fn foo() {}
}
impl S {
    fn bar() { }
}
impl T for S {
    fn foo() {
        Self::bar();
    }
}