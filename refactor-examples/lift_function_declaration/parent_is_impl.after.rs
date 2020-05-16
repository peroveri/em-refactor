fn main() { }

struct S;
impl S {
    fn foo() {
        
        Self::bar();
    }
    fn bar() { }
}