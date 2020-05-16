fn main() { }

struct S;
trait T {
    fn foo() {}
}
impl T for S {
    fn foo() {
        
        Self::bar();
    }
}
impl S {
    fn bar() { }
}