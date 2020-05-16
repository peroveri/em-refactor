fn main() { }

struct S;
trait T {
    fn foo(&self) {}
}
impl S {
    fn bar(&self) { }
}
impl T for S {
    fn foo(&self) {
        self.bar();
    }
}