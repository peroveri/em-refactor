fn main() { }

struct S;
trait T {
    fn foo(&self) {}
}
impl T for S {
    fn foo(&self) {
        fn bar(self_: &S) { }
        bar(self);
    }
}