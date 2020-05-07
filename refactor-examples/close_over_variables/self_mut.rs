fn main() { }
impl S {
    fn foo(&mut self) {
        (|| {
            self.bar();
        })();
    }
    fn bar(&mut self) { }
}
struct S;