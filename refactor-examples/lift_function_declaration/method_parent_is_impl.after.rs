fn main() { }

struct S;
impl S {
    fn foo(&self) {
        
        self.bar();
    }
    fn bar(&self) { }
}