fn main() { }
impl S {
    fn foo(&mut self) {
        (|self_: &mut S| {
            (*self_).bar();
        })(self);
    }
    fn bar(&mut self) { }
}
struct S;