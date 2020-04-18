struct S(i32);
fn main() {
    let i = S(0);
    let j = &i.0;
    j;
}
// Extract line 3-5 should fail as a borrow is used later