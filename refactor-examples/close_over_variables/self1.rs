fn main() { }
impl S {
    fn foo(&self) {
        (|| {
            self.bar();
        })();
    }
    fn bar(&self) { }
}
struct S;