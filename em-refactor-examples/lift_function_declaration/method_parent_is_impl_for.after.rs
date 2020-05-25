fn main() { }

struct S;
trait T {
    fn foo(&self) {}
}
impl T for S {
    fn foo(&self) {
        
        self.bar();
    }
}
impl S {
    fn bar(&self) { }
}