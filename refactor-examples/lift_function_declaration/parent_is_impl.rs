fn main() { }

struct S;
impl S {
    fn foo() {
        fn bar() { }
        bar();
    }
}