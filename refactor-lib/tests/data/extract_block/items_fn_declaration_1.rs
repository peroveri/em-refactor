fn main() {
    fn foo() {}
    foo();
}
// after extracting the function declaration at line 2, it will no longer be
// visible at line 3