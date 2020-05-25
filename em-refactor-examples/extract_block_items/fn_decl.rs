fn foo() {panic!("");}
fn main() {
    foo();
    fn foo() {}
    foo();
}