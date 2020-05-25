fn main() { }

struct S;
impl S {
    fn foo(&self) {
        fn bar(self_: &S) { }
        bar(self);
    }
}