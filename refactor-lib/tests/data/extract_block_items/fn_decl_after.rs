fn foo() {panic!("");}
fn main() {
    fn foo() {}
    {foo();}
    foo();
}