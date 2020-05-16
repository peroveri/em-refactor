fn main() { }

struct S;
impl S {
    fn foo(&mut self) {
        fn bar(self_: &mut S) { }
        bar(self);
    }
}