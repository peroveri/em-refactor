fn main() { }

struct S;
impl S {
    fn bar() { }
    fn foo() {
        Self::bar();
    }
}