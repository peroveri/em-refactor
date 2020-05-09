fn foo(s: &Box<i32>) {
    (|| {
        b(s);
    })();
}
fn b(s: &i32) {}
fn main() {}
// An example where the current refactoring implementation crashes
// b(s) at line 3 borrows s without using the '&'
// Fixed