fn main() { }
impl S {
    fn foo(&self) {
        (|self_: &crate::S| {
            self_.bar();
        })(self);
    }
    fn bar(&self) { }
}
struct S;