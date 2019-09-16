fn new_fn(i: &mut i32) {
    *i += 1
}
fn foo() {
    let mut i = 1;
    new_fn(&mut i)
}
