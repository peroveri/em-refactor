fn main() { }
impl S {
    fn foo(&mut self) {
        (|self_: &mut crate::S| {
            self_.bar();
        })(self);
    }
    fn bar(&mut self) { }
}
struct S;