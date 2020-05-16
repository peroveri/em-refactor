fn main() { }

struct S;
impl S {
    fn foo(&mut self) {
        
        self.bar();
    }
    fn bar(&mut self) { }
}