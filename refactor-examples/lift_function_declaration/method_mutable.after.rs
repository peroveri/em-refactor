fn main() { }

struct S;
impl S {
    fn bar(&mut self) { }
    fn foo(&mut self) {
        self.bar();
    }
}